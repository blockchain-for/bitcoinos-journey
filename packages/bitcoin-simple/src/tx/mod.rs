pub mod model;

use std::time::{SystemTime, UNIX_EPOCH};

pub use model::*;

use crate::crypto::{self, key};

pub fn create_signed(
    keypair: &crypto::KeyPair,
    to: key::PublicKey,
    amount: u32,
) -> SignedTransaction {
    let from = keypair.public_key;
    let created_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    let tx = Transaction {
        tx_id: Transaction::generate_tx_id(from, to, amount, created_at),
        from,
        to,
        amount,
        created_at,
    };

    let sig = keypair.sign(&tx.hash());

    SignedTransaction {
        transaction: tx,
        sig: sig.to_string(),
    }
}
