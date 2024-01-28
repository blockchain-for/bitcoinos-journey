pub mod key;

use std::str::FromStr;

use secp256k1::{ecdsa::Signature, rand, All, Message, Secp256k1};
use sha2::{Digest, Sha256};

pub fn sha256(payload: String) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(payload.as_bytes());
    hasher.finalize().to_vec()
}

pub struct KeyPair {
    secp: Secp256k1<All>,
    pub public_key: key::PublicKey,
    pub private_key: key::SecretKey,
}

impl KeyPair {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from(key: String) -> Result<Self, secp256k1::Error> {
        let secp = Secp256k1::new();
        let private_key = key::SecretKey::from_str(&key)?;
        let public_key = key::PublicKey::from_secret_key(&secp, &private_key);

        Ok(Self {
            secp,
            public_key,
            private_key,
        })
    }
    pub fn sign(&self, message: &[u8]) -> Signature {
        let message = Message::from_digest_slice(message).expect("Invalid message");
        self.secp.sign_ecdsa(&message, &self.private_key)
    }
}

impl Default for KeyPair {
    fn default() -> Self {
        let mut rand = rand::thread_rng();

        let secp = Secp256k1::new();
        let (private_key, public_key) = secp.generate_keypair(&mut rand);
        Self {
            secp,
            public_key,
            private_key,
        }
    }
}
