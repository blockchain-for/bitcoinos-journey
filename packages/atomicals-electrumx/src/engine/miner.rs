use std::path::PathBuf;

use bitcoin::Network;

use crate::{electrumx::ElectrumX, AnyhowResult};

use super::{tx::TransactionData, wallet::Wallet};

pub struct Miner {
    pub network: Network,
    pub electrumx: ElectrumX,
    pub wallets: Vec<Wallet>,
    ticker: String,
    max_fee: u64,
}

impl Miner {
    pub async fn mine(&self, wallet: &Wallet) -> AnyhowResult<()> {
        let data = self.prepare_data(wallet).await?;

        // Construct tx data
        todo!()
    }

    pub async fn prepare_data(&self, wallet: &Wallet) -> AnyhowResult<TransactionData> {
        todo!()
    }
}

pub struct MinerBuilder {
    pub network: Network,
    pub electrumx: String,
    pub wallet_dir: PathBuf,
    pub ticker: String,
    pub max_fee: u64,
}

impl MinerBuilder {
    pub fn new(
        network: Network,
        electrumx: impl Into<String>,
        wallet_dir: impl Into<String>,
        ticker: impl Into<String>,
        max_fee: u64,
    ) -> Self {
        Self {
            network,
            electrumx: electrumx.into(),
            wallet_dir: wallet_dir.into().into(),
            ticker: ticker.into(),
            max_fee,
        }
    }

    pub fn build(&self) -> AnyhowResult<Miner> {
        todo!()
    }
}
