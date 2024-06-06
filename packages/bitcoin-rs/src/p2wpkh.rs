//! from: https://github.com/panicfarm/multisig-test/blob/master/examples/sighash.rs
//!
use bitcoin::Amount;

/// Computes SegWit sighash for a transaction input that spends a p2wpkh output with "witness_v0_keyhash" scriptPubKey.type
///
/// # Arguments
///
/// * `raw_tx` - spending tx hex
/// * `inp_idx` - spending tx input index
/// * `value` - ref tx output value in sats
pub fn verify_signature(mut raw_tx: &[u8], input_idx: usize, value: u64) {
    // Step 1: Decode the transaction from raw bytes
    let tx: bitcoin::Transaction =
        bitcoin::consensus::Decodable::consensus_decode(&mut raw_tx).unwrap();

    // Step 2: Get the input from the transaction with the given index
    let input = &tx.input[input_idx];

    // Step 3: Get the witness from TxIn
    let witness = &input.witness;
    println!("Witness: {:?}", witness);

    // Check the witness length must be 2
    // BIP-141: The witness must be consist of exactly 2 items (<= 520 bytes each).
    // The first one is a signature, and the second one is a public key.
    assert_eq!(witness.len(), 2);

    // Step 4: Get signature hash & public key bytes from witness
    let sighash_bytes = witness.nth(0).unwrap();
    let pk_bytes = witness.nth(1).unwrap();

    // Step 5: translate signature hash bytes into signature
    let signature =
        bitcoin::ecdsa::Signature::from_slice(sighash_bytes).expect("Failed to parse sighash");

    // Step 6: translate public key bytes into public key and WPubkeyHash
    // // BIP 143: The Item 5 : For P2WPKH witness program, the scriptCode is `0x1976a914{20-byte-pubkey-hash}88a`
    // // this is nothing but a standard P2PKH script OP_DUP OP_HASH160 <pubKeyHash> OP_EQUALVERIFY OP_CHECKSIG
    let pk = bitcoin::PublicKey::from_slice(pk_bytes).expect("Failed to parse pubkey");
    let wpkh = pk.wpubkey_hash().expect("compressed key");
    println!("Script pubkey hash: {wpkh:x}");

    // Step 7: Build P2WPKH ScriptBuf from WPubkeyHash
    let spk = bitcoin::ScriptBuf::new_p2wpkh(&wpkh);
    // let script_code = spk.p2wpkh_script_code().expect("Failed to get script code");

    // Step 8: Build SighashCache for the transaction
    let mut cache = bitcoin::sighash::SighashCache::new(&tx);
    let sighash = cache
        .p2wpkh_signature_hash(
            input_idx,
            // &script_code,
            &spk,
            Amount::from_sat(value),
            signature.sighash_type,
        )
        .expect("Failed to compute sighash");

    println!("Segwit p2wpkh sighash: {sighash:x}");

    // verify this
    let msg = bitcoin::secp256k1::Message::from_digest_slice(&sighash[..]).unwrap();
    println!("Message is: {msg:x}");

    let secp = bitcoin::secp256k1::Secp256k1::new();
    secp.verify_ecdsa(&msg, &signature.signature, &pk.inner)
        .unwrap();
}
