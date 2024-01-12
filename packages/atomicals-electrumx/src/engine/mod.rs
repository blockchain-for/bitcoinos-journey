use std::path::Path;

use bitcoin::Network;

use crate::model::AnyhowResult;

pub async fn run(
    network: Network,
    electrumx: &str,
    wallet_dir: &Path,
    ticker: &str,
    max_fee: u64,
) -> AnyhowResult<()> {
    todo!()
}
