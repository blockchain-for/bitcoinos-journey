use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};

use bitcoind::{
    node::Node,
    p2p,
    settings::{self, ENV_PREFIX},
};

use colored::*;

fn main() -> std::io::Result<()> {
    let config = settings::Settings::new("config.yml", ENV_PREFIX)
        .unwrap()
        .config;
    println!("Settings: {config:#?}");

    let data_dir = config.data_dir;

    let p2p_data = p2p::server::P2pData::default();
    let p2p_data_arc = Arc::new(Mutex::new(p2p_data));

    // Broadcast blocks and transactions
    let (block_tx, block_rx) = mpsc::channel();
    let (transaction_tx, transaction_rx) = mpsc::channel();

    // Interrupt the miner when new blocks are received throught the network
    let (miner_interrupt_tx, miner_interrupt_rx) = mpsc::channel();

    let receiver_p2p_data_arc = p2p_data_arc.clone();
    let receiver_thread = thread::spawn(move || {
        p2p::run_receiver(receiver_p2p_data_arc, block_rx, transaction_rx).unwrap();
    });

    // Start Node
    let node = Node::new(block_tx, transaction_tx, &data_dir);
    let node_arc = Arc::new(Mutex::new(node));
    {
        let mut node_instance = node_arc.lock().unwrap();
        node_instance.start().expect("Started Bitcoind failed");
        println!(
            "{}",
            format!("Your public key: {}", node_instance.keypair.public_key).yellow()
        );
    }

    // // Start RPC
    // let rpc_node_clone = node_arc.clone();
    // let rpc_port = config.rpc_port.clone();
    // let rpc_thread = thread::spawn(move|| {
    //     rpc::run_server(rpc_node_clone, rpc_port)
    // });

    // // Start P2P
    let p2p_node_clone = node_arc.clone();
    let tcp_port = config.tcp_port.clone();
    let server_p2p_data_clone = p2p_data_arc.clone();
    let host_addr = format!("{}:{}", config.host_ip, tcp_port);
    let run_server_host_addr = host_addr.clone();

    let p2p_miner_interrupt_tx = miner_interrupt_tx.clone();
    let p2p_data_dir = data_dir.clone();
    let p2p_thread = thread::spawn(move || {
        p2p::run(
            p2p_node_clone.clone(),
            server_p2p_data_clone,
            run_server_host_addr,
            p2p_miner_interrupt_tx,
            &p2p_data_dir,
        )
        .unwrap();
    });

    // Start Miner
    // let p2p_node_miner = node_arc.clone();
    // let miner_thread = if config.miner_enabled {
    //     let miner_node_clone = node_arc.clone();
    //     Some(thread::spawn(move || {
    //         miner::start_miner(miner_node_clone, miner_interrupt_rx)
    //     }))
    // } else { None };

    // Init p2p
    let p2p_node_clone = node_arc.clone();
    let p2p_data_clone = p2p_data_arc.clone();
    let init_host_addr = &host_addr;
    let p2p_data_dir = data_dir.clone();
    p2p::init(
        p2p_node_clone.clone(),
        p2p_data_clone,
        miner_interrupt_tx,
        init_host_addr,
        config.bootstrap_nodes,
        &p2p_data_dir,
    )
    .unwrap();

    // // Web
    // let web_port = config.web_port.clone();
    // let web_thread = thread::spawn(move || {
    //     web::run_server(web_port);
    // });

    // // Join threads
    receiver_thread.join().unwrap();
    // rpc_thread.join().unwrap();
    p2p_thread.join().unwrap();
    // if let Some(t) = miner_thread {
    //     t.join().unwrap();
    // }
    // web_thread.join().unwrap();

    Ok(())
}
