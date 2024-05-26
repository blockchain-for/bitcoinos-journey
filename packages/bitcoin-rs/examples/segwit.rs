use bitcoin::{
    absolute, ecdsa,
    secp256k1::{Message, Secp256k1},
    sighash::SighashCache,
    Amount, EcdsaSighashType, Network, ScriptBuf, Sequence, TxIn, TxOut, Witness,
};
use bitcoin_rs::{receivers_address, senders_keys, unspent_transaction_output};

const DUMMY_UTXO_AMOUNT: u64 = 20_000_000;
const SPEND_AMOUNT: u64 = 5_000_000;
const CHANGE_AMOUNT: u64 = 14_900_000;

fn main() {
    let address = "tb1pngqsfevnf6l7g8zx9mazf533wrtvt5zp44zg0j9g8vdcuy97dfuqywc5nf";
    // let address = "bc1q7cyrfmck2ffu2ud3rn5l5a8yv6f0chkp0zpemf";
    let network = Network::Testnet;
    // let network = Network::Bitcoin;
    let secp = Secp256k1::new();
    let (sk, wpkh) = senders_keys(&secp);
    let address = receivers_address(address, network);

    let spend_amount = Amount::from_sat(DUMMY_UTXO_AMOUNT);

    let (dummy_out_point, dummy_utxo) = unspent_transaction_output(&wpkh, spend_amount);

    // The script code required to spend a p2wpkh output
    let script_code = dummy_utxo
        .script_pubkey
        .p2wpkh_script_code()
        .expect("Must be a valid script");

    // The input for the transaction
    let input = TxIn {
        previous_output: dummy_out_point,
        script_sig: ScriptBuf::new(),
        sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
        witness: Witness::default(),
    };

    // The spend output is locked to a key controlled by the receiver
    let spend = TxOut {
        value: Amount::from_sat(SPEND_AMOUNT),
        script_pubkey: address.script_pubkey(),
    };

    let change = TxOut {
        value: Amount::from_sat(CHANGE_AMOUNT),
        script_pubkey: ScriptBuf::new_p2wpkh(&wpkh),
    };

    // build the transaction
    let unsigned_tx = bitcoin::Transaction {
        version: bitcoin::transaction::Version(2),
        lock_time: absolute::LockTime::ZERO,
        input: vec![input],
        output: vec![spend, change],
    };

    // Sign the unsigned transaction
    let mut signhash_cache = SighashCache::new(unsigned_tx);
    let sighash = signhash_cache
        .p2wsh_signature_hash(
            0,
            &script_code,
            spend_amount,
            bitcoin::EcdsaSighashType::All,
        )
        .expect("Must be a valid sighash");

    let msg = Message::from(sighash);
    let signature = secp.sign_ecdsa(&msg, &sk);

    // Convert into a transaction
    let mut tx = signhash_cache.into_transaction();

    // Update the witness stack
    let pk = sk.public_key(&secp);
    let witness = &mut tx.input[0].witness;
    witness.push_ecdsa_signature(&ecdsa::Signature {
        sig: signature,
        hash_ty: EcdsaSighashType::All,
    });

    witness.push(pk.serialize());

    println!("segwit tx: {tx:?}");
}
