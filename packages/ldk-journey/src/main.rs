
use ldk_node::bitcoin::secp256k1::PublicKey;
use ldk_node::{Builder, NetAddress};
use ldk_node::bitcoin::Network;

use std::str::FromStr;

fn main() {
    // let mut config = Config::default();
    // config.network = Network::Testnet;

    // let mut builder = Builder::from_config(config);
    let mut builder = Builder::new();

    // let esplora_server_url = "http://ldk-node.tnull.de:3002".to_string();
    // builder.set_esplora_server(esplora_server_url);
    builder.set_esplora_server("https://blockstream.info/testnet/api".to_string());
	builder.set_gossip_source_rgs("https://rapidsync.lightningdevkit.org/testnet/snapshot".to_string());
    builder.set_network(Network::Regtest);

    let node = builder.build().unwrap();

    node.start().unwrap();

    println!("ADDRESS: {:?}", node.new_onchain_address().unwrap());

    let node_id = node.node_id();
    println!("NODE ID: {:?}", node_id);
    println!("ONCHAIN FUNDS: {}", node.spendable_onchain_balance_sats().unwrap());

    let node_id = PublicKey::from_str(node_id.to_string().as_str()).unwrap();
    let node_addr = NetAddress::from_str("blockstream.info:80").unwrap();

    let open_res = node.connect_open_channel(node_id, node_addr, 1000, None, None, false);
    println!("CHANNEL OPEN: {:?}", open_res);

    let event = node.wait_next_event();
    println!("EVENT: {:?}", event);
    node.event_handled();

    let event = node.wait_next_event();
    println!("EVENT: {:?}", event);
    node.event_handled();

    node.stop().unwrap();
}
