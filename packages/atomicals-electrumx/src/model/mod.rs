pub mod request;
pub mod response;

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Params<P>
where
    P: Serialize,
{
    pub params: P,
}

impl<P> Params<P>
where
    P: Serialize,
{
    pub fn new(params: P) -> Self {
        Self { params }
    }
}

/// message is some when success is false, and data is none, otherwise data is some and message is none
#[derive(Debug, Deserialize)]
pub struct Response<R> {
    pub success: bool,
    // pub message: Option<String>,
    pub response: R,
}

#[derive(Debug, Deserialize)]
pub struct GlobalResponse<R> {
    pub global: Option<Global>,
    pub result: R,
}

#[derive(Debug, Deserialize)]
pub struct Global {
    pub atomical_count: u64,
    pub atomicals_block_hashes: HashMap<String, String>,
    pub atomicals_block_tip: String,
    pub block_tip: String,
    pub coin: String,
    pub height: u64,
    pub network: String,
    pub server_time: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Ticker {
    pub status: String,
    pub candidate_atomical_id: String,
    pub atomical_id: String,
    pub candidates: Vec<Candidate>,
    pub r#type: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Candidate {
    pub tx_num: u64,
    pub atomical_id: String,
    pub commit_height: u64,
    pub reveal_location_height: u64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Ft {
    #[serde(rename = "$bitwork")]
    pub bitwork: Bitwork,
    #[serde(rename = "$max_mints")]
    pub max_mints: u64,
    #[serde(rename = "$max_supply")]
    pub max_supply: u64,
    #[serde(rename = "$mint_amount")]
    pub mint_amount: u64,
    #[serde(rename = "$mint_bitcworkc")]
    pub mint_bitworkc: String,
    #[serde(rename = "$mint_height")]
    pub mint_height: u64,
    #[serde(rename = "$request_ticker")]
    pub request_ticker: String,
    #[serde(rename = "$request_ticker_status")]
    pub request_ticker_status: TickerStatus,
    #[serde(rename = "$ticker")]
    pub ticker: String,
    #[serde(rename = "$ticker_candidates")]
    pub ticker_candidates: Vec<TickerCandidate>,

    pub atomical_id: String,
    pub atomical_number: u64,
    pub atomical_ref: String,
    pub confirmed: bool,
    pub dft_info: DftInfo,
    pub location_summary: LocationSummary,
    pub mint_data: MintData,
    pub mint_info: MintInfo,
    pub subtype: String,
    pub r#tpye: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Bitwork {
    pub bitworkc: String,
    pub bitworkr: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TickerStatus {
    pub note: String,
    pub status: String,
    pub verified_atomical_id: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TickerCandidate {
    pub atomical_id: String,
    pub commit_height: u64,
    pub reveal_location_height: u64,
    pub tx_num: u64,
    pub txid: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DftInfo {
    pub mint_count: u64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LocationSummary {
    pub circulating_supply: u64,
    pub unique_holders: u64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MintData {
    pub fields: Fields,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Fields {
    pub args: Args,
    pub meta: Option<Meta>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Args {
    pub bitworkc: String,
    pub max_mints: u64,
    pub mint_amount: u64,
    pub mint_bitworkc: String,
    pub mint_height: u64,
    // TODO: It's a `String` in mainnet but a `u64` in testnet.
    pub nonce: u64,
    pub request_ticker: String,
    pub time: u64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Meta {
    pub description: Option<String>,
    pub lega: Option<Legal>,
    pub name: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Legal {
    pub terms: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MintInfo {
    #[serde(rename = "$bitwork")]
    pub bitwork: Bitwork,
    #[serde(rename = "$bitworkc")]
    pub bitworkc: String,
    #[serde(rename = "$request_ticker")]
    pub request_ticker: String,
    pub args: Args,
    pub commit_height: u64,
    pub commit_index: u64,
    pub commit_location: String,
    pub commit_tx_num: u64,
    pub commit_txid: String,
    pub ctx: Ctx,
    pub meta: Meta,
    pub reveal_location: String,
    pub reveal_location_blockhash: String,
    pub reveal_location_header: String,
    pub reveal_location_height: u64,
    pub reveal_location_index: u64,
    pub reveal_location_script: String,
    pub reveal_location_scripthash: String,
    pub reveal_location_tx_num: u64,
    pub reveal_location_txid: String,
    pub reveal_location_value: u64,
}

// TODO: Check the real type.
#[derive(Debug, Deserialize, Serialize)]
pub struct Ctx {}

#[derive(Debug, Deserialize, Serialize)]
pub struct Unspent {
    pub txid: String,
    pub tx_hash: String,
    pub index: u32,
    pub tx_pos: u32,
    pub vout: u32,
    pub height: u64,
    pub value: u64,
    // TODO: Check the real type.
    pub atomicals: Vec<()>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Utxo {
    pub txid: String,
    // The same as `output_index` and `index`.
    pub vout: u32,
    pub value: u64,
    // TODO: check type
    pub atomicals: Vec<()>,
}

impl From<Unspent> for Utxo {
    fn from(v: Unspent) -> Self {
        Self {
            txid: v.tx_hash,
            vout: v.tx_pos,
            value: v.value,
            atomicals: v.atomicals,
        }
    }
}
