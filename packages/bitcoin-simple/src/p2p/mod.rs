pub mod server;

use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use colored::Colorize;

use crate::block::Block;
use crate::node::Node;
use crate::storage;
use crate::tx::SignedTransaction;

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
    block_rx: mpsc::Receiver<Block>,
    transaction_rx: mpsc::Receiver<SignedTransaction>,
) -> ResultUnit {
    let mut now = Instant::now();

    loop {
        // TODO: keep trying to reconnect to bootstrap nodes if they go offline
        if now.elapsed().as_secs() > 60 {
            check_and_update_peers(p2p_data.clone())?;
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

fn publish_transaction(p2p_data: Arc<Mutex<P2pData>>, tx: SignedTransaction) -> ResultUnit {
    publish(
        p2p_data,
        MESSAGE_NEW_TRANSACTION,
        serde_json::to_string(&tx)?,
    )
}

fn publish_block(p2p_data: Arc<Mutex<P2pData>>, block: Block) -> ResultUnit {
    publish(p2p_data, MESSAGE_NEW_BLOCK, serde_json::to_string(&block)?)
}

pub fn publish(p2p_data: Arc<Mutex<P2pData>>, req: &str, data: String) -> ResultUnit {
    let p2p_data = p2p_data.lock().unwrap();
    for peer in &p2p_data.peers {
        if let Err(e) = send_message(peer, req.to_owned(), Some(data.clone())) {
            println!(
                "{}",
                format!(
                    "Failed to publish {} to {}, happened error: {e:?}",
                    data, peer
                )
                .red()
            );
        }
    }

    Ok(())
}

pub fn add_peer(
    node: Arc<Mutex<Node>>,
    data: Arc<Mutex<P2pData>>,
    miner_interrupt_tx: mpsc::Sender<()>,
    remote_peer: &str,
    data_dir: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut data = data.lock().unwrap();
    if !data.peers.iter().any(|x| x == remote_peer) {
        println!("{} {} added", "New Peer:".green(), remote_peer);
        data.peers.push(remote_peer.to_owned());
        check_peer_blocks(node, miner_interrupt_tx, remote_peer, data_dir)?;
    }

    Ok(())
}

pub fn check_and_update_peers(data: Arc<Mutex<P2pData>>) -> ResultUnit {
    let mut data = data.lock().unwrap();
    data.peers.retain(|peer| {
        if let Err(e) = send_message(peer, MESSAGE_PING.to_string(), None) {
            println!("Disconnected from peer: {} - {:?}", peer, e);
            false
        } else {
            true
        }
    });

    Ok(())
}

pub fn send_message(
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

pub fn check_peer_blocks(
    node: Arc<Mutex<Node>>,
    miner_interrupt_tx: mpsc::Sender<()>,
    peer: &str,
    data_dir: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Get blocks from remote peer
    let blocks_resp = send_message(peer, MESSAGE_GET_BLOCKS.to_string(), None)?;
    let block_hashes: Vec<String> = serde_json::from_str(&blocks_resp)?;

    // Get latest block hash from storage
    let missing_hashes =
        match storage::get_latest_block_hash(&storage::db::blocks_metadata(true, data_dir))? {
            Some(latest_block_hash) => {
                let position = block_hashes.iter().position(|b| *b == latest_block_hash);
                match position {
                    Some(position) => block_hashes[position + 1..].to_vec(),
                    None => vec![],
                }
            }
            None => block_hashes,
        };

    for block_hash in missing_hashes {
        let block = send_message(peer, MESSAGE_GET_BLOCK.to_owned(), Some(block_hash))?;
        let block: Block = serde_json::from_str(&block)?;
        let mut node = node.lock().unwrap();
        node.process_block(&block)?;
        miner_interrupt_tx.send(())?;
    }

    Ok(())
}

pub fn check_peers(data: Arc<Mutex<P2pData>>) -> ResultUnit {
    let mut data = data.lock().unwrap();

    data.peers.retain(|peer| {
        if let Err(e) = send_message(peer, MESSAGE_PING.to_string(), None) {
            println!("Disconnected from peer: {} - {:?}", peer, e);
            false
        } else {
            true
        }
    });

    Ok(())
}

pub fn init(
    node: Arc<Mutex<Node>>,
    data: Arc<Mutex<P2pData>>,
    miner_interrupt_tx: mpsc::Sender<()>,
    host_addr: &str,
    bootstrap_nodes: Vec<String>,
    data_dir: &str,
) -> ResultUnit {
    bootstrap_nodes.iter().for_each(|peer| {
        if let Err(e) = init_node(
            node.clone(),
            data.clone(),
            miner_interrupt_tx.clone(),
            peer,
            host_addr,
            data_dir,
        ) {
            println!("Failed to add peer: {e}");
        }
    });

    Ok(())
}

pub fn init_node(
    node: Arc<Mutex<Node>>,
    data: Arc<Mutex<P2pData>>,
    miner_interrupt_tx: mpsc::Sender<()>,
    remote_peer: &str,
    host_addr: &str,
    data_dir: &str,
) -> ResultUnit {
    add_peer(
        node.clone(),
        data.clone(),
        miner_interrupt_tx.clone(),
        remote_peer,
        data_dir,
    )?;

    let resp = send_message(
        remote_peer,
        MESSAGE_NEW_PEER.to_string(),
        Some(host_addr.to_string()),
    )?;
    let peers: Vec<String> = serde_json::from_str(&resp)?;

    for peer in peers {
        add_peer(
            node.clone(),
            data.clone(),
            miner_interrupt_tx.clone(),
            &peer,
            data_dir,
        )?;
        send_message(
            remote_peer,
            MESSAGE_NEW_PEER.to_string(),
            Some(host_addr.to_string()),
        )?;
    }
    Ok(())
}
