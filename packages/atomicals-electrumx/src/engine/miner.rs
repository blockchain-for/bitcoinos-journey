use std::path::PathBuf;

use bitcoin::Network;

use crate::{electrumx::ElectrumX, wallet::Wallet, AnyhowResult};

use super::{tx::TransactionData, wallet::EngineWallet};

pub struct Miner {
    pub network: Network,
    pub electrumx: ElectrumX,
    pub wallets: Vec<EngineWallet>,
    ticker: String,
    max_fee: u64,
}

impl Miner {
    pub async fn mine(&self, wallet: &EngineWallet) -> AnyhowResult<()> {
        let data = self.prepare_data(wallet).await?;

        let reveal_spk = todo!();
        let funding_spk = todo!();

        let commit_input = todo!();
        let commit_ouput = todo!();

        // Construct tx data
        todo!()
    }

    pub async fn prepare_data(&self, wallet: &EngineWallet) -> AnyhowResult<TransactionData> {
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
        let electrumx = ElectrumX::new(self.network, self.electrumx.as_str(), 30)?;
        let wallets = Wallet::load_wallets(self.wallet_dir.as_path())
            .into_iter()
            .map(|rw| EngineWallet::from_raw_wallet(rw, self.network))
            .collect::<Result<_>>()?;

        Ok(Miner {
            network: self.network,
            electrumx,
            wallets,
            ticker: self.ticker,
            max_fee: self.max_fee,
        })
    }
}
