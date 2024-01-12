pub mod miner;
pub mod tx;
pub mod wallet;

use bitcoin::Network;

use crate::{engine::miner::MinerBuilder, AnyhowResult};

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

    Ok(())
}
