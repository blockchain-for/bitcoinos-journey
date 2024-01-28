use std::fmt::{Debug, Display};
use std::str::FromStr;

use secp256k1::{ecdsa::Signature, Message, PublicKey, Secp256k1};
use serde::{Deserialize, Serialize};

use crate::crypto;

#[derive(Clone, Copy, Deserialize, Serialize)]
pub struct Transaction {
    pub from: PublicKey,
    pub to: PublicKey,
    pub amount: u32,
}

impl Debug for Transaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Transaction")
            .field("from", &format!("{}", self.from))
            .field("to", &format!("{}", self.to))
            .field("amount", &self.amount)
            .finish()
    }
}

impl Transaction {
    pub fn serialize(&self) -> String {
        format!(
            "{}{}{}",
            self.from,
            self.to,
            hex::encode(format!("{}", self.amount))
        )
    }

    pub fn hash(&self) -> Vec<u8> {
        crypto::sha256(self.serialize())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SignedTransaction {
    pub transaction: Transaction,
    pub sig: String,
}

impl Display for SignedTransaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.transaction.serialize(), self.sig)
    }
}

impl SignedTransaction {
    pub fn is_sig_valid(&self) -> bool {
        let secp = Secp256k1::verification_only();
        let unsigned_tx_hash = Message::from_digest_slice(self.transaction.hash().as_slice())
            .expect("Message must valid");
        let sig = Signature::from_str(self.sig.as_str()).expect("Signature must valid");
        secp.verify_ecdsa(&unsigned_tx_hash, &sig, &self.transaction.from)
            .is_ok()
    }
}
