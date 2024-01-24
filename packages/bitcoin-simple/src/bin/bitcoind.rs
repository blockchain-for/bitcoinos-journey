use std::{sync::{mpsc, Arc, Mutex}, thread};

use bitcoind::settings;

use colored::*;

fn main() -> std::io::Result<()> {
    let config = settings::Settings::new().unwrap();

    let p2p_data = p2p::P2pData::new();
    let p2p_data_arc = Arc::new(Mutex::new(p2p_data));

    // Broadcast blocks and transactions
    let (block_tx, block_rx) = mpsc::channel();
    let (transaction_tx, transaction_rx) = mpsc::channel();

    // Interrupt the miner when new blocks are received throught the network
    let (miner_interrupt_tx, miner_interrupt_rx) = mpsc::channel();

    let receiver_p2p_data_arc = p2p_data_arc.clone();
    let receiver_thread = thread::spawn(move || {
        p2p::run_receiver(receiver_p2p_data_arc, block_rx, transaction_rx);
    });

    // Start Node
    let node = Node::(block_tx, transaction_tx);
    let node_arc = Arc::new(Mutex::new(node));
    {
        let mut node_instance = node_arc.lock().unwrap();
        node_instance.start().expect("Started Bitcoind failed");
        println!("{}", format!("Your public key: {}", node_instance.keypair.public_key).yellow());
    }

    // Start RPC
    let rpc_node_clone = node_arc.clone();
    let rpc_port = config.rpc_port.clone();
    let rpc_thread = thread::spawn(move|| {
        rpc::run_server(rpc_node_clone, rpc_port)
    });

    // Start P2P
    let p2p_node_clone = node_arc.clone();
    let tcp_port = config.tcp_port.clone();
    let server_p2p_data_clone = p2p_data_arc.clone();
    let host_addr= format!("{}:{}", config.host_ip, tcp_port);
    let run_server_host_addr = host_addr.clone();
    
    
    Ok(())
}