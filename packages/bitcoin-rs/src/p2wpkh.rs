use bitcoin::Amount;


/// Computes SegWit sighash for a transaction input that spends a p2wpkh output with "witness_v0_keyhash" scriptPubKey.type
/// 
/// # Arguments
/// 
/// * `raw_tx` - spending tx hex
/// * `inp_idx` - spending tx input index
/// * `value` - ref tx output value in sats
pub fn verify_signature(
    mut raw_tx: &[u8],
    input_idx: usize,
    value: u64,
) {
    let tx: bitcoin::Transaction = bitcoin::consensus::Decodable::consensus_decode(&mut raw_tx).unwrap();
    
    let input = &tx.input[input_idx];
    let witness = &input.witness;
    println!("Witness: {witness:?}");

    // BIP-141: The witness must be consist of exactly 2 items (<= 520 bytes each).
    // The first one is a signature, and the second one is a public key.
    assert_eq!(witness.len(), 2);

    let sighash_bytes = witness.nth(0).unwrap();
    let pk_bytes = witness.nth(1).unwrap();

    let signature = bitcoin::ecdsa::Signature::from_slice(sighash_bytes).expect("Failed to parse sighash");

    // BIP 143: The Item 5 : For P2WPKH witness program, the scriptCode is `0x1976a914{20-byte-pubkey-hash}88a`
    // this is nothing but a standard P2PKH script OP_DUP OP_HASH160 <pubKeyHash> OP_EQUALVERIFY OP_CHECKSIG
    let pk = bitcoin::PublicKey::from_slice(pk_bytes).expect("Failed to parse pubkey");
    let wpkh = pk.wpubkey_hash().expect("compressed key");
    println!("Script pubkey hash: {wpkh:x}");

    let spk = bitcoin::ScriptBuf::new_p2wpkh(&wpkh);
    let script_code = spk.p2wpkh_script_code().expect("Failed to get script code");

    let mut cache = bitcoin::sighash::SighashCache::new(&tx);
    let sighash = cache.p2wpkh_signature_hash(input_idx, &script_code, Amount::from_sat(value), signature.sighash_type)
        .expect("Failed to compute sighash");

    println!("Segwit p2wpkh sighash: {sighash:x}");

    // verify this
    let msg = bitcoin::secp256k1::Message::from_slice(&sighash[..]).unwrap();
    println!("Message is: {msg:x}");

    let secp = bitcoin::secp256k1::Secp256k1::new();
    secp.verify_ecdsa(&msg, &signature.signature, &pk.inner).unwrap();

}