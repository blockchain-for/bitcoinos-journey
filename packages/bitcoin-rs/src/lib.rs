use std::str::FromStr;

use bitcoin::{
    hashes::Hash,
    secp256k1::{rand, Secp256k1, SecretKey, Signing},
    Address, Amount, Network, OutPoint, ScriptBuf, TxOut, Txid, WPubkeyHash,
};

pub fn senders_keys<C: Signing>(secp: &Secp256k1<C>) -> (SecretKey, WPubkeyHash) {
    let sk = SecretKey::new(&mut rand::thread_rng());
    let pk = bitcoin::PublicKey::new(sk.public_key(secp));
    let wpkh = pk.wpubkey_hash().expect("Key is compressed");

    (sk, wpkh)
}

pub fn receivers_address(address: &str, network: Network) -> Address {
    Address::from_str(address)
        .expect("Must be a valid Bitcoin Address")
        .require_network(network)
        .expect("Address must match network")
}

pub fn unspent_transaction_output(wpkh: &WPubkeyHash, amount: Amount) -> (OutPoint, TxOut) {
    let script_pubkey = ScriptBuf::new_p2wpkh(wpkh);

    let out_point = OutPoint {
        txid: Txid::all_zeros(),
        vout: 0,
    };

    let utxo = TxOut {
        value: amount,
        script_pubkey,
    };

    (out_point, utxo)
}
