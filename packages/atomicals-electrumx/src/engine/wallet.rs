use bitcoin::{secp256k1::Keypair, Address, XOnlyPublicKey};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct Wallet {
    pub stash: Key,
    pub funding: Key,
}

#[derive(Debug, Clone)]
pub struct Key {
    pair: Keypair,
    x_only_public_key: XOnlyPublicKey,
    address: Address,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PayloadWrapper {
    pub args: Payload,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Payload {
    pub bitworkc: String,
    pub mint_ticker: String,
    pub nonce: u64,
    pub time: u64,
}
