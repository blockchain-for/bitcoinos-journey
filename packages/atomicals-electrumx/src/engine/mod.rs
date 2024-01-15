pub mod miner;
pub mod tx;
pub mod wallet;

use bitcoin::{Network, TxOut};

use crate::{engine::miner::MinerBuilder, AnyhowResult};

use self::tx::Fees;

const BASE_BYTES: f64 = 10.5;
const INPUT_BYTES_BASE: f64 = 57.5;
const OUTPUT_BYTES_BASE: f64 = 43.;
const REVEAL_INPUT_BYTES_BASE: f64 = 66.;

/// Entry of engine, engine as a miner to mint atomicals assets
pub async fn run(
    network: Network,
    electrumx: &str,
    wallet_dir: &str,
    ticker: &str,
    max_fee: u64,
) -> AnyhowResult<()> {
    // create Miner
    let miner = MinerBuilder::new(network, electrumx, wallet_dir, ticker, max_fee).build()?;

    // #[alow(clippy::never_loop)]
    loop {
        for w in &miner.wallets {
            miner.mine(w).await?
        }
    }
}

pub fn fees_of(satsbyte: u64, reveal_script_len: usize, additional_outputs: &[TxOut]) -> Fees {
    let satsbyte = satsbyte as f64;

    let commit = (satsbyte * (BASE_BYTES + INPUT_BYTES_BASE + OUTPUT_BYTES_BASE).ceil()) as u64;

    let reveal = {
        let compact_input_bytes = if reveal_script_len <= 252 {
            1.
        } else if reveal_script_len <= 0xFFFF {
            3.
        } else if reveal_script_len <= 0xFFFFFFFF {
            5.
        } else {
            9.
        };

        (satsbyte
            * (BASE_BYTES
                + REVEAL_INPUT_BYTES_BASE
                + (compact_input_bytes + reveal_script_len as f64) / 4.
                + additional_outputs.len() as f64 * OUTPUT_BYTES_BASE))
            .ceil() as u64
    };

    let outputs = additional_outputs
        .iter()
        .map(|o| o.value.to_sat())
        .sum::<u64>();
    let commit_and_reveal = commit + reveal;
    let commit_and_reveal_and_outputs = commit_and_reveal + outputs;

    Fees {
        commit,
        commit_and_reveal_and_outputs,
        reveal_and_outputs: reveal + outputs,
    }
}
