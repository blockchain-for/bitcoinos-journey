pub mod legacy_multisig;
pub mod p2wpkh;
pub mod p2wsh_multisig_22;
pub mod pltc;
pub mod psbt0;

pub mod scripts;
pub mod tx;
pub mod varint;
pub mod version;

use std::str::FromStr;

use bitcoin::{
    hashes::Hash,
    key::UntweakedPublicKey,
    script::Instruction,
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

/// Decode M of N multisig ScriptPubKey into a required signatures count M and a vector pubkeys of length N
///
/// # Arguments
///
/// * `script_pubkey` - p2wsh multisig scriptPubKey
pub fn decode_script_pubkey(script_pubkey: &bitcoin::Script) -> (usize, Vec<bitcoin::PublicKey>) {
    println!("ScriptePubKey: {:?}", script_pubkey);
    println!(
        "Instructions len: {:?}",
        script_pubkey.instructions().count()
    );

    let mut pubkey_vec = vec![];
    let mut pubkey_cnt = 0;
    let mut required_sig_cnt = 0;

    for (k, instr) in script_pubkey.instructions().enumerate() {
        match instr.unwrap() {
            Instruction::PushBytes(pb) => {
                println!("The index {k} is PushBytes: {pb:?}");
                assert!(k > 0);
                let pk = bitcoin::PublicKey::from_slice(pb.as_bytes()).unwrap();
                pubkey_vec.push(pk);
            }
            Instruction::Op(op) => {
                println!("The index {k} is Op: {op:?}");
                if k == 0 {
                    required_sig_cnt =
                        match op.classify(bitcoin::blockdata::opcodes::ClassifyContext::Legacy) {
                            bitcoin::blockdata::opcodes::Class::PushNum(m) => m,
                            _ => panic!("NaN"),
                        };
                } else if op == bitcoin::blockdata::opcodes::all::OP_CHECKMULTISIG {
                    assert!(
                        pubkey_vec.len() == pubkey_cnt.try_into().unwrap(),
                        "{}: {} -- pubkey vec len {}, pubkey cnt {}",
                        k,
                        op,
                        pubkey_vec.len(),
                        pubkey_cnt,
                    );

                    println!(
                        "ScriptPubKey is {}x{} MULTISIG",
                        required_sig_cnt, pubkey_cnt
                    );
                } else {
                    assert!(k == pubkey_vec.len() + 1);

                    pubkey_cnt =
                        match op.classify(bitcoin::blockdata::opcodes::ClassifyContext::Legacy) {
                            bitcoin::blockdata::opcodes::Class::PushNum(n) => n,
                            _ => panic!("NaN"),
                        };

                    assert!(pubkey_vec.len() == pubkey_cnt.try_into().unwrap());
                }
            }
        }
    }

    (required_sig_cnt.try_into().unwrap(), pubkey_vec)
}
