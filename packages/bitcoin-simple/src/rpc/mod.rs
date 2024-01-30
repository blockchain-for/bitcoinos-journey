mod model;

use std::sync::{Arc, Mutex};

use colored::Colorize;
use jsonrpc_core::IoHandler;
use jsonrpc_http_server::ServerBuilder;

use crate::{node::Node, p2p::ResultUnit, rpc::model::RpcInstance};

use jsonrpc_core::Result;
use jsonrpc_derive::rpc;

use crate::{block::Block, crypto, tx::SignedTransaction};

#[rpc(server, client)]
pub trait Rpc {
    #[rpc(name = "protocolVersion")]
    fn protocol_version(&self) -> Result<String>;

    #[rpc(name = "send")]
    fn send(&self, pubkey: crypto::key::PublicKey, amount: u32) -> Result<SignedTransaction>;

    #[rpc(name = "newpubkey")]
    fn newpubkey(&self) -> Result<String>;

    #[rpc(name = "getpubkey")]
    fn getpubkey(&self) -> Result<String>;

    #[rpc(name = "blockheight")]
    fn blockheight(&self) -> Result<u32>;

    #[rpc(name = "getblock")]
    fn getblock(&self, block_number: u32) -> Result<Option<Block>>;

    #[rpc(name = "balances")]
    fn balances(&self) -> Result<std::collections::HashMap<crypto::key::PublicKey, u32>>;

    #[rpc(name = "getbalance")]
    fn getbalance(&self, pubkey: crypto::key::PublicKey) -> Result<u32>;

    #[rpc(name = "mempool")]
    fn mempool(&self) -> Result<Vec<SignedTransaction>>;
}

pub fn run_server(node: Arc<Mutex<Node>>, host: String, port: u32) -> ResultUnit {
    let mut io = IoHandler::new();
    let rpc = RpcInstance::new(node);
    io.extend_with(rpc.to_delegate());

    let rpc_path = format!("{host}:{port}");

    let server = ServerBuilder::new(io)
        .threads(3)
        .start_http(&rpc_path.parse()?)?;

    println!("{} Listening on {:?}:{:?}\n", "RPC".green(), host, rpc_path);

    Ok(server.wait())
}
