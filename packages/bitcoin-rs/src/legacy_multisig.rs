//! from https://github.com/panicfarm/multisig-test/blob/master/examples/sighash.rs
//!

use bitcoin::sighash;

use crate::decode_script_pubkey;

/// Computes sighash for a legacy multisig transaction input that spends either a p2sh or a p2ms output
///
/// # Arguments
///
/// * `raw_tx` - spending tx hex
/// * `input_idx` - spending tx input index
/// * `script_pubkey_bytes_opt` - Option with scriptPubKey bytes.
/// *  - If None, it's p2sh case, i.e. reftx output's scriptPubKey.type is "scripthash". In this case scriptPubkey is extracted from the spending transaction's scritSig.
/// *  - If Some(), it's p2ms case, i.e. reftx output's scriptPubKey.type is "multisig", and the scriptPubkey is supplied from the referenced output.
pub fn verify_signature_legacy(
    mut raw_tx: &[u8],
    input_idx: usize,
    script_pubkey_bytes_opt: Option<&[u8]>,
) {
    let tx: bitcoin::Transaction =
        bitcoin::consensus::Decodable::consensus_decode(&mut raw_tx).unwrap();

    let input = &tx.input[input_idx];
    let script_sig = &input.script_sig;
    println!("scriptSig is: {script_sig}");

    let sighasher = sighash::SighashCache::new(&tx);

    // In the P2SH case we get scriptPubKey from scriptSig of the spending input.
    // The scriptSig that corresponds to an M of N multisig should be: PUSHBYTES_0 PUSHBYTES_K0 <sig0><sighashflag0> ... PUSHBYTES_Km <sigM><sighashflagM> PUSHBYTES_X <scriptPubKey>
    // Here we assume that we have an M of N multisig scriptPubKey.
    let mut instructions: Vec<_> = script_sig.instructions().collect();
    let script_pubkey_p2sh;

    let script_pubkey_bytes = match script_pubkey_bytes_opt {
        // In the P2MS case, the scriptPubkey is in refereneced output, passed into this function
        Some(bytes) => bytes,
        // In the P2SH case, the scriptPubkey is the last scriptSig PushBytes instruction
        None => {
            script_pubkey_p2sh = instructions.pop().unwrap().unwrap();
            script_pubkey_p2sh.push_bytes().unwrap().as_bytes()
        }
    };

    let script_code = bitcoin::Script::from_bytes(script_pubkey_bytes);

    // For a M of N multisig, the required_sig_cnt will be M and pubkey_vec.len() is N:
    let (required_sig_cnt, pubkey_vec) = decode_script_pubkey(script_code);

    let n = pubkey_vec.len();

    let pushbytes_0 = instructions.remove(0).unwrap();

    assert!(
        pushbytes_0.push_bytes().unwrap().as_bytes().is_empty(),
        "first in ScriptSig must be PUSHBYTES_0 got {:?}",
        pushbytes_0,
    );

    let mut sig_verified_cnt = 0;

    // All other scriptSig instructions must be signatures
    for instr in instructions {
        let sig =
            bitcoin::ecdsa::Signature::from_slice(instr.unwrap().push_bytes().unwrap().as_bytes())
                .expect("failed to parse signature");
        let sighash = sighasher
            .legacy_signature_hash(input_idx, script_code, sig.sighash_type.to_u32())
            .expect("failed to compute sighash");

        println!(
            "Legacy sighash: {:x} (sighash flag {})",
            sighash, sig.sighash_type
        );

        let msg = bitcoin::secp256k1::Message::from_digest_slice(&sighash[..]).unwrap();

        for pk in &pubkey_vec {
            let secp = bitcoin::secp256k1::Secp256k1::new();
            match secp.verify_ecdsa(&msg, &sig.signature, &pk.inner) {
                Ok(_) => {
                    sig_verified_cnt += 1;
                    println!("Verified signature with PubKey {}", pk);
                }
                Err(err) => println!("{}", err),
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
        "verified {} signatures for {} of {} multisig\n\n",
        sig_verified_cnt, required_sig_cnt, n
    );
}
