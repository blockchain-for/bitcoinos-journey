use std::io::{Read, Write};
use std::{
    net::{TcpListener, TcpStream},
    sync::{mpsc, Arc, Mutex},
};

use colored::Colorize;
use regex::Regex;

use crate::node::Node;
use crate::storage;

const MESSAGE_NEW_PEER: &str = "NEW_PEER";
const MESSAGE_PING: &str = "PING";

const MESSAGE_GET_BLOCK: &str = "GET_BLOCK";
const MESSAGE_GET_BLOCKS: &str = "GET_BLOCKS";

const MESSAGE_NEW_BLOCK: &str = "NEW_BLOCK";
const MESSAGE_NEW_TRANSACTION: &str = "NEW_TRANSACTION";

pub type ResultUnit = core::result::Result<(), Box<dyn std::error::Error>>;

#[derive(Debug, Clone, Default)]
pub struct P2pData {
    pub peers: Vec<String>,
}

#[derive(Debug, Clone)]
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

    pub fn serve(&mut self) -> ResultUnit {
        let listener = TcpListener::bind(self.host_addr.as_str())?;

        println!(
            "{:?} Listening on {:?}",
            "P2p".green(),
            listener.local_addr()?.to_string()
        );

        for stream in listener.incoming() {
            self.handle_connection(stream?)?;
        }

        Ok(())
    }

    pub fn handle_connection(&mut self, mut stream: TcpStream) -> ResultUnit {
        let mut buffer = [0; 100000];

        stream.read_exact(&mut buffer)?;

        let msg = String::from_utf8_lossy(&buffer[..]);
        let msg = msg.as_ref();

        let response = self.response(msg)?;

        stream.write_all("Hello, client!".as_bytes())?;
        stream.flush()?;

        Ok(())
    }

    fn response(&mut self, msg: &str) -> Result<String, String> {
        if msg.starts_with(MESSAGE_PING) {
            Ok(String::from("OK"))
        } else if msg.starts_with(MESSAGE_GET_BLOCKS) {
            self.handle_get_blocks()
        } else if msg.starts_with(MESSAGE_GET_BLOCK) {
            let regex = Regex::new(&format!(r"{}\((?P<hash>.*?)\)", MESSAGE_GET_BLOCK)).unwrap();
            let caps = regex.captures(msg).unwrap();
            let block = &caps["hash"];
            self.handle_get_block(block)
        } else if msg.starts_with(MESSAGE_NEW_BLOCK) {
            let regex = Regex::new(&format!(r"{}\((?P<block>.*?)\)", MESSAGE_GET_BLOCK)).unwrap();
            let caps = regex.captures(msg).unwrap();
            let block = &caps["block"];
            self.handle_get_block(block)
        } else if msg.starts_with(MESSAGE_NEW_PEER) {
            let re = Regex::new(&format!(r"{}\((?P<host>.*?)\)", MESSAGE_NEW_PEER))?;
            let caps = re.captures(msg).unwrap();
            let host = &caps["host"];
            self.handle_new_peer(host.to_string())
        } else if msg.starts_with(MESSAGE_NEW_TRANSACTION) {
            let re = Regex::new(&format!(r"{}\((?P<tx>.*?)\)", MESSAGE_NEW_TRANSACTION))?;
            let caps = re.captures(msg).unwrap();
            let tx = &caps["tx"];
            self.handle_new_transaction(tx.to_string())
        } else {
            Err(String::from("Invalid MESSAGE"))
        }
    }

    fn handle_get_blocks(&mut self) -> Result<String, String> {
        let block_hashes = storage::get_block_hashes(&storage::db::blocks_md(true))?;
        Ok(serde_json::to_string(&block_hashes).unwrap())
    }

    fn handle_get_block(&mut self, block_hash: &str) -> Result<String, String> {
        todo!()
    }

    fn handle_new_block(&mut self, block: String) -> Result<String, String> {
        todo!()
    }
}

pub fn run(
    node: Arc<Mutex<Node>>,
    data: Arc<Mutex<P2pData>>,
    host_addr: impl Into<String>,
    miner_interrupt_tx: mpsc::Sender<()>,
) -> ResultUnit {
    let mut server = P2pServer::new(node, data, host_addr, miner_interrupt_tx);

    server.serve()?;

    Ok(())
}

pub fn run_receiver(
    p2p_data: Arc<Mutex<P2pData>>,
    block_rx: mpsc::Receiver<()>,
    transaction_rx: mpsc::Receiver<()>,
) -> ResultUnit {
    Ok(())
}
