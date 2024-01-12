use bitcoin::Address;
use sha2::{Digest, Sha256};

use crate::model::AnyhowResult;

pub fn address2scripthash(address: &Address) -> AnyhowResult<String> {
    let mut hasher = Sha256::new();
    hasher.update(address.script_pubkey());

    let mut hash = hasher.finalize();
    hash.reverse();

    Ok(array_bytes::bytes2hex("", hash))
}
