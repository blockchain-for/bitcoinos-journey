use std::path::PathBuf;

use bitcoin::{secp256k1::Secp256k1, taproot::TaprootBuilder, Amount, Network, TxOut};

use crate::{
    electrumx::{Api, ElectrumX},
    engine::tx::{Payload, PayloadWrapper},
    model::Ft,
    utils,
    wallet::Wallet,
    AnyhowResult,
};

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
        let ft = self.validate().await?;

        let secp = Secp256k1::new();
        let satsbyte = if self.network == Network::Bitcoin {
            utils::query_fee().await?.min(self.max_fee) + 5
        } else {
            5
        };

        let additional_outputs = vec![TxOut {
            value: Amount::from_sat(ft.mint_amount),
            script_pubkey: wallet.stash.address.script_pubkey(),
        }];

        let payload = PayloadWrapper {
            args: {
                let (time, nonce) = utils::time_nonce();

                Payload {
                    bitworkc: ft.mint_bitworkc.clone(),
                    mint_ticker: ft.ticker.clone(),
                    nonce,
                    time,
                }
            },
        };

        let payload_encoded = utils::cbor(&payload)?;

        let reveal_script =
            utils::build_reveal_script(&wallet.funding.x_only_public_key, "dmt", &payload_encoded);

        let reveal_spend_info = TaprootBuilder::new()
            .add_leaf(0, reveal_script.clone())?
            .finalize(&secp, wallet.funding.x_only_public_key)
            .unwrap();

        let fees = super::fees_of(
            satsbyte,
            reveal_script.as_bytes().len(),
            &additional_outputs,
        );

        let funding_utxo = self
            .electrumx
            .wait_util_utxo(
                wallet.funding.address.to_string(),
                fees.commit_and_reveal_and_outputs,
            )
            .await?;

        Ok(TransactionData {
            secp,
            satsbyte,
            bitwork_info_commit: ft.mint_bitworkc,
            additional_outputs,
            reveal_script,
            reveal_spend_info,
            fees,
            funding_utxo,
        })
    }

    async fn validate(&self) -> AnyhowResult<Ft> {
        let id = self
            .electrumx
            .get_by_ticker(&self.ticker)
            .await?
            .atomical_id;
        let response = self.electrumx.get_ft_info(id).await?;
        let global = response.global.unwrap();
        let ft = response.result;

        if ft.ticker != self.ticker {
            Err(anyhow::anyhow!("ticker mismatch"))?;
        }
        if ft.subtype != "decentralized" {
            Err(anyhow::anyhow!("not decentralized"))?;
        }
        if ft.mint_height > global.height + 1 {
            Err(anyhow::anyhow!("mint height mismatch"))?;
        }
        if ft.mint_amount == 0 || ft.mint_amount >= 100_000_000 {
            Err(anyhow::anyhow!("mint amount mismatch"))?;
        }

        Ok(ft)
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
            .collect::<AnyhowResult<_>>()?;

        Ok(Miner {
            network: self.network,
            electrumx,
            wallets,
            ticker: self.ticker.clone(),
            max_fee: self.max_fee,
        })
    }
}
