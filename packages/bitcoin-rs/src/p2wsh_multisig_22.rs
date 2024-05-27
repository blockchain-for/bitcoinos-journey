//! from https://github.com/panicfarm/multisig-test/blob/master/examples/sighash.rs
//!
use bitcoin::{sighash, Amount};

use crate::decode_script_pubkey;

/// Computes sighash for a segwit multisig transaction input that spends a p2wsh output with `witness_v0_scripthash` scriptPubKey.type
///
/// # Arguments
/// * `raw_tx` - spending tx hex
/// * `input_idx` - spending tx input index
/// * `value` - ref tx output value in sats
pub fn verify_signature(mut raw_tx: &[u8], input_idx: usize, value: u64) {
    let tx: bitcoin::Transaction =
        bitcoin::consensus::Decodable::consensus_decode(&mut raw_tx).unwrap();
    let input = &tx.input[input_idx];
    let witness = &input.witness;
    println!("witness: {witness:?}");

    // the last element is called witnessScript according to BIP-141. It supersedes scriptPubkey
    let witness_script_bytes = witness.last().expect("Out of bounds");
    let witness_script = bitcoin::Script::from_bytes(witness_script_bytes);

    let mut cache = sighash::SighashCache::new(&tx);

    // For a M of N multisig, the required_sig_cnt will be M and pubkey_vec.len() is N:
    let (required_sig_cnt, pubkey_vec) = decode_script_pubkey(witness_script);
    let n = pubkey_vec.len();

    let mut sig_verified_cnt = 0;

    println!("Starting build sighash cache");
    // In an M of N multisig, the witness elements from 1 (0-based) to M-2 are signatures (with sighash flags as the last byte)
    for i in 1..=witness.len() - 2 {
        let sig_bytes = witness.nth(i).expect("Out of bounds");
        let sig = bitcoin::ecdsa::Signature::from_slice(sig_bytes).expect("Failed to parse sig");

        let sig_len = sig_bytes.len() - 1;

        // Last byte is EcdsaSighashType sighash flag
        // ECDSA signature in DER format lengths are between 70 and 72 bytes
        assert!(
            (70..=72).contains(&sig_len),
            "signature length {} out of bounds",
            sig_len
        );

        // Here we assume that all sighash_flags are the same. Can they be different?
        let sighash = cache
            .p2wsh_signature_hash(
                input_idx,
                witness_script,
                Amount::from_sat(value),
                sig.sighash_type,
            )
            .expect("Failed to compute sighash");

        println!("Segwit p2wsh sighash: {:x} ({})", sighash, sig.sighash_type);

        let msg = bitcoin::secp256k1::Message::from_digest_slice(&sighash[..]).unwrap();

        for pk in &pubkey_vec {
            let secp = bitcoin::secp256k1::Secp256k1::new();

            match secp.verify_ecdsa(&msg, &sig.signature, &pk.inner) {
                Ok(_) => {
                    sig_verified_cnt += 1;
                    println!("Verified signature with PubKey {}", pk);
                }
                Err(e) => println!("{}", e),
            }
        }
    }

    // test
    assert!(
        sig_verified_cnt == required_sig_cnt,
        "{} signatures verified out of {} expected",
        sig_verified_cnt,
        required_sig_cnt
    );

    println!(
        "verified {} signatures for {} of {} multisig\n",
        sig_verified_cnt, required_sig_cnt, n,
    );
}
