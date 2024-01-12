use bitcoin::{
    secp256k1::{All, Secp256k1},
    taproot::TaprootSpendInfo,
    ScriptBuf, TxOut,
};

use crate::model::Utxo;

#[derive(Clone, Debug)]
pub struct TransactionData {
    pub secp: Secp256k1<All>,
    pub satsbyte: u64,
    pub bitwork_info: String,
    pub additional_outputs: Vec<TxOut>,
    pub reveal_script: ScriptBuf,
    pub reveal_spend_info: TaprootSpendInfo,
    pub fees: Fees,
    pub funding_utxo: Utxo,
}

#[derive(Clone, Debug)]
pub struct Fees {
    pub commit: u64,
    pub commit_and_reveal_and_outputs: u64,
    pub reveal_and_outputs: u64,
}
