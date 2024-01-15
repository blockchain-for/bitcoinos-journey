use std::{
    ops::Range,
    time::{SystemTime, UNIX_EPOCH},
};

use bitcoin::{
    opcodes::{
        all::{OP_CHECKSIG, OP_ENDIF, OP_IF},
        OP_0,
    },
    script::PushBytes,
    secp256k1::Keypair,
    Address, PrivateKey, Script, ScriptBuf, TxOut, XOnlyPublicKey,
};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::AnyhowResult;

pub fn address2scripthash(address: &Address) -> AnyhowResult<String> {
    let mut hasher = Sha256::new();
    hasher.update(address.script_pubkey());

    let mut hash = hasher.finalize();
    hash.reverse();

    Ok(array_bytes::bytes2hex("", hash))
}

pub fn keypair_from_wif<S>(wif: S) -> AnyhowResult<Keypair>
where
    S: AsRef<str>,
{
    Ok(Keypair::from_secret_key(
        &Default::default(),
        &PrivateKey::from_wif(wif.as_ref())?.inner,
    ))
}

pub fn sequence_ranges_by_cpus(max: u32) -> Vec<Range<u32>> {
    let step = (max as f64 / num_cpus::get() as f64).ceil() as u32;

    let mut ranges = vec![];

    let mut start = 0;

    while start < max {
        let end = start.checked_add(step).unwrap_or(max);
        ranges.push(start..end);
        start = end;
    }

    ranges
}

pub async fn query_fee() -> AnyhowResult<u64> {
    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct FastestFee {
        fastest_fee: u64,
    }

    Ok(
        reqwest::get("https://mempool.space/api/v1/fees/recommended")
            .await?
            .json::<FastestFee>()
            .await?
            .fastest_fee,
    )
}

pub fn time_nonce() -> (u64, u64) {
    (
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        rand::thread_rng().gen_range(1..10_000_000),
    )
}

pub fn cbor<T>(v: &T) -> AnyhowResult<Vec<u8>>
where
    T: Serialize,
{
    let mut cbor = Vec::new();

    ciborium::into_writer(v, &mut cbor)?;

    Ok(cbor)
}

pub fn build_reveal_script(
    x_only_public_key: &XOnlyPublicKey,
    op_type: &str,
    payload: &[u8],
) -> ScriptBuf {
    // format!(
    // 	"{} OP_CHECKSIG OP_0 OP_IF {} {} {} OP_ENDIF",
    // 	&private_key.public_key(&Default::default()).to_string()[2..],
    // 	array_bytes::bytes2hex("", "atom"),
    // 	array_bytes::bytes2hex("", op_type),
    // 	payload.chunks(520).map(|c| array_bytes::bytes2hex("", c)).collect::<Vec<_>>().join(" ")
    // )
    let script_builder = Script::builder()
        .push_x_only_key(x_only_public_key)
        .push_opcode(OP_CHECKSIG)
        .push_opcode(OP_0)
        .push_opcode(OP_IF)
        .push_slice(<&PushBytes>::try_from("atom".as_bytes()).unwrap())
        .push_slice(<&PushBytes>::try_from(op_type.as_bytes()).unwrap());

    payload
        .chunks(520)
        .fold(script_builder, |b, c| b.push_slice(<&PushBytes>::try_from(c).unwrap()))
        .push_opcode(OP_ENDIF)
        .into_script()
}
