use bitcoin::{
    absolute,
    key::{TapTweak, TweakedKeypair},
    secp256k1::{Message, Secp256k1},
    sighash::{Prevouts, SighashCache},
    transaction::Version,
    Amount, Network, ScriptBuf, Sequence, TapSighashType, Transaction, TxIn, TxOut, Witness,
};
use bitcoin_rs::{keypair, receivers_address, taproot_tx_output};

const DUMMY_UTXO_AMOUNT: u64 = 20_000_000;
const SPEND_AMOUNT: u64 = 5_000_000;
const CHANGE_AMOUNT: u64 = 14_900_000;

fn main() {
    let address = "tb1pngqsfevnf6l7g8zx9mazf533wrtvt5zp44zg0j9g8vdcuy97dfuqywc5nf";
    let network = Network::Testnet;
    let secp = Secp256k1::new();

    let keypair = keypair(&secp);
    let (internal_key, _parity) = keypair.x_only_public_key();

    let address = receivers_address(address, network);
    let amount = Amount::from_sat(DUMMY_UTXO_AMOUNT);

    let (outpoint, tx_out) = taproot_tx_output(&secp, internal_key, amount);

    let input_tx = TxIn {
        previous_output: outpoint,
        script_sig: ScriptBuf::new(),
        sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
        witness: Witness::default(),
    };

    let spend = TxOut {
        value: Amount::from_sat(SPEND_AMOUNT),
        script_pubkey: address.script_pubkey(),
    };

    let change = TxOut {
        value: Amount::from_sat(CHANGE_AMOUNT),
        script_pubkey: ScriptBuf::new_p2tr(&secp, internal_key, None),
    };

    let unsigned_tx = Transaction {
        version: Version(2),
        lock_time: absolute::LockTime::ZERO,
        input: vec![input_tx],
        output: vec![spend, change],
    };

    // create prevous output
    let prevouts = vec![tx_out];
    let prevouts = Prevouts::All(&prevouts);

    // Sign the unsigned transaction
    let mut sighash_cache = SighashCache::new(&unsigned_tx);
    let sighash = sighash_cache
        .taproot_signature_hash(0, &prevouts, None, None, TapSighashType::Default)
        .expect("Muste be taproot signature");
    let tweaked: TweakedKeypair = keypair.tap_tweak(&secp, None);

    let msg = Message::from(sighash);
    let sig = secp.sign_schnorr(&msg, &tweaked.to_inner());

    let tx = sighash_cache.into_transaction();

    let mut witness = tx.input[0].witness.clone();
    witness.push(sig.as_ref());

    println!("tx: {tx:?}");
}
