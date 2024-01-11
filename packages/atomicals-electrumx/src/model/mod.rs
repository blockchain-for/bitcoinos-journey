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
    pub message: Option<String>,
    pub data: Option<R>,
}

#[derive(Debug, Deserialize)]
pub struct GlobalResponse<R> {
    pub global: Option<Global>,
    pub data: R,
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
