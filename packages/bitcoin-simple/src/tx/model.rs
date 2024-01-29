use std::fmt::{Debug, Display};
use std::str::FromStr;

use secp256k1::{ecdsa::Signature, Message, PublicKey, Secp256k1};
use serde::{Deserialize, Serialize};

use crate::crypto;

#[derive(Clone, Deserialize, Serialize)]
pub struct Transaction {
    pub tx_id: String,
    pub from: PublicKey,
    pub to: PublicKey,
    pub amount: u32,
    pub created_at: u64,
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
        Self::generate_tx_id(self.from, self.to, self.amount, self.created_at)
    }

    pub fn hash(&self) -> Vec<u8> {
        crypto::sha256(self.serialize())
    }

    pub fn generate_tx_id(from: PublicKey, to: PublicKey, amount: u32, created_at: u64) -> String {
        format!(
            "{}{}{}{}",
            from,
            to,
            hex::encode(format!("{}", amount)),
            hex::encode(format!("{}", created_at)),
        )
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

    pub fn tx_id(&self) -> String {
        self.transaction.tx_id.clone()
    }
}
