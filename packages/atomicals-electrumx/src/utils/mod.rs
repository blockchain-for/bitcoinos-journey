use bitcoin::{secp256k1::Keypair, Address, PrivateKey};
use sha2::{Digest, Sha256};

use crate::AnyhowResult;

pub fn address2scripthash(address: &Address) -> AnyhowResult<String> {
    let mut hasher = Sha256::new();
    hasher.update(address.script_pubkey());

    let mut hash = hasher.finalize();
    hash.reverse();

    Ok(array_bytes::bytes2hex("", hash))
}

pub fn keypair_from_wif<S>(wif: S) -> AnyhowResult<Keypair>
where
    S: AsRef<str>,
{
    Ok(Keypair::from_secret_key(
        &Default::default(),
        &PrivateKey::from_wif(wif.as_ref())?.inner,
    ))
}
