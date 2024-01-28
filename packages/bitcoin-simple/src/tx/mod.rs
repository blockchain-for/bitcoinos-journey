pub mod model;

pub use model::*;

use crate::crypto::{self, key};

pub fn create_signed(
    keypair: &crypto::KeyPair,
    to: key::PublicKey,
    amount: u32,
) -> SignedTransaction {
    let tx = Transaction {
        from: keypair.public_key,
        to,
        amount,
    };

    let sig = keypair.sign(&tx.hash());

    SignedTransaction {
        transaction: tx,
        sig: sig.to_string(),
    }
}
