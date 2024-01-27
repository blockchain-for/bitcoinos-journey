pub mod server;

use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use colored::Colorize;

use crate::node::Node;

use self::server::{P2pData, P2pServer};

pub type ResultUnit = core::result::Result<(), Box<dyn std::error::Error>>;

const MESSAGE_NEW_PEER: &str = "NEW_PEER";
const MESSAGE_PING: &str = "PING";

const MESSAGE_GET_BLOCK: &str = "GET_BLOCK";
const MESSAGE_GET_BLOCKS: &str = "GET_BLOCKS";

const MESSAGE_NEW_BLOCK: &str = "NEW_BLOCK";
const MESSAGE_NEW_TRANSACTION: &str = "NEW_TRANSACTION";

pub fn run(
    node: Arc<Mutex<Node>>,
    data: Arc<Mutex<P2pData>>,
    host_addr: impl Into<String>,
    miner_interrupt_tx: mpsc::Sender<()>,
    data_dir: &str,
) -> ResultUnit {
    let mut server = P2pServer::new(node, data, host_addr, miner_interrupt_tx);

    server.serve(data_dir)?;

    Ok(())
}

pub fn run_receiver(
    p2p_data: Arc<Mutex<P2pData>>,
    block_rx: mpsc::Receiver<()>,
    transaction_rx: mpsc::Receiver<()>,
) -> ResultUnit {
    let mut now = Instant::now();

    loop {
        // TODO: keep trying to reconnect to bootstrap nodes if they go offline
        if now.elapsed().as_secs() > 60 {
            check_peers(p2p_data.clone())?;
            now = Instant::now();
        }

        if let Ok(block) = block_rx.try_recv() {
            publish_block(p2p_data.clone(), block)?;
        }

        if let Ok(tx) = transaction_rx.try_recv() {
            publish_transaction(p2p_data.clone(), tx)?;
        }

        thread::sleep(Duration::from_millis(100));
    }
}

fn publish_transaction(clone: Arc<Mutex<P2pData>>, tx: ()) -> ResultUnit {
    todo!()
}

fn publish_block(clone: Arc<Mutex<P2pData>>, block: ()) -> ResultUnit {
    todo!()
}

pub fn add_peer(
    node: Arc<Mutex<Node>>,
    data: Arc<Mutex<P2pData>>,
    miner_interrupt_tx: mpsc::Sender<()>,
    addr: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut data = data.lock().unwrap();
    if !data.peers.iter().any(|x| x == addr) {
        println!("{} {}", "New Peer:".green(), addr);
        data.peers.push(addr.to_owned());
        check_peer_block(node, miner_interrupt_tx, addr)?;
    }
    todo!()
}

pub fn check_peers(data: Arc<Mutex<P2pData>>) -> ResultUnit {
    let mut data = data.lock().unwrap();
    data.peers.retain(|peer| {
        if let Err(e) = send(peer, MESSAGE_PING.to_string(), None) {
            println!("Disconnected from peer: {} - {:?}", peer, e);
            false
        } else {
            true
        }
    });

    Ok(())
}

pub fn send(
    addr: &str,
    message: String,
    data: Option<String>,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut stream = TcpStream::connect(addr)?;

    let msg = data
        .map(|d| format!("{}({})", message, d))
        .unwrap_or(message);

    stream.write_all(msg.as_bytes())?;

    let mut buffer = [0; 100000];
    stream.read_exact(&mut buffer)?;

    let resp = String::from_utf8_lossy(&buffer[..]);
    let resp_parts: Vec<_> = resp.split("\r\n").collect();
    let resp = resp_parts[0].to_string();

    Ok(resp)
}

pub fn check_peer_block(
    node: Arc<Mutex<Node>>,
    miner_interrupt_tx: mpsc::Sender<()>,
    peer: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    todo!()
}
