
pub mod p2wpkh;

use std::str::FromStr;

use bitcoin::{
    hashes::Hash,
    key::UntweakedPublicKey,
    secp256k1::{rand, Keypair, Secp256k1, SecretKey, Signing, Verification},
    Address, Amount, Network, OutPoint, ScriptBuf, TxOut, Txid, WPubkeyHash,
};

pub fn senders_keys<C: Signing>(secp: &Secp256k1<C>) -> (SecretKey, WPubkeyHash) {
    let sk = SecretKey::new(&mut rand::thread_rng());
    let pk = bitcoin::PublicKey::new(sk.public_key(secp));
    let wpkh = pk.wpubkey_hash().expect("Key is compressed");

    (sk, wpkh)
}

pub fn keypair<C: Signing>(secp: &Secp256k1<C>) -> Keypair {
    let sk = SecretKey::new(&mut rand::thread_rng());
    Keypair::from_secret_key(secp, &sk)
}

pub fn receivers_address(address: &str, network: Network) -> Address {
    Address::from_str(address)
        .expect("Must be a valid Bitcoin Address")
        .require_network(network)
        .expect("Address must match network")
}

pub fn unspent_transaction_output(wpkh: &WPubkeyHash, amount: Amount) -> (OutPoint, TxOut) {
    let script_pubkey = ScriptBuf::new_p2wpkh(wpkh);

    init_tx_output(script_pubkey, amount)
}

pub fn taproot_tx_output<C: Verification>(
    secp: &Secp256k1<C>,
    internal_key: UntweakedPublicKey,
    amount: Amount,
) -> (OutPoint, TxOut) {
    let script_pubkey = ScriptBuf::new_p2tr(secp, internal_key, None);

    init_tx_output(script_pubkey, amount)
}

pub fn init_tx_output(script_pubkey: ScriptBuf, amount: Amount) -> (OutPoint, TxOut) {
    let out_point = OutPoint {
        txid: Txid::all_zeros(),
        vout: 0,
    };

    let tx_out = TxOut {
        value: amount,
        script_pubkey,
    };

    (out_point, tx_out)
}
