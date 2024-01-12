use std::{str::FromStr, time::Duration};

use bitcoin::{Address, Network};
use serde::{de::DeserializeOwned, Serialize};
use tokio::time;

use crate::{
    model::{Ft, GlobalResponse, Params, Response, Ticker, Unspent, Utxo},
    utils, AnyhowResult,
};

pub trait Config {
    fn network(&self) -> &Network;
    fn base_uri(&self) -> &str;
}

pub trait Http {
    async fn post<U, P, R>(&self, uri: U, params: P) -> AnyhowResult<R>
    where
        U: AsRef<str>,
        P: Serialize,
        R: DeserializeOwned;
}

pub trait Api: Config + Http {
    fn uri_of<S>(&self, uri: S) -> String
    where
        S: AsRef<str>,
    {
        format!("{}/{}", self.base_uri(), uri.as_ref())
    }

    async fn get_by_ticker<S>(&self, ticker: S) -> AnyhowResult<Ticker>
    where
        S: AsRef<str>,
    {
        self.post::<_, _, Response<GlobalResponse<Ticker>>>(
            self.uri_of("blockchain.atomicals.get_by_ticker"),
            Params::new([ticker.as_ref()]),
        )
        .await
        .map(|d| d.response.result)
    }

    async fn get_ft_info<S>(&self, atomical_id: S) -> AnyhowResult<GlobalResponse<Ft>>
    where
        S: AsRef<str>,
    {
        self.post::<_, _, Response<GlobalResponse<Ft>>>(
            self.uri_of("blockchain.atomicals.get_ft_info"),
            Params::new([atomical_id.as_ref()]),
        )
        .await
        .map(|r| r.response)
    }

    async fn get_unspent_scripthash<S>(&self, scripthash: S) -> AnyhowResult<Vec<Utxo>>
    where
        S: AsRef<str>,
    {
        let mut utxos = self
            .post::<_, _, Response<Vec<Unspent>>>(
                self.uri_of("blockchain.scripthash.listupsent"),
                Params::new([scripthash.as_ref()]),
            )
            .await?
            .response
            .into_iter()
            .map(|u| u.into())
            .collect::<Vec<Utxo>>();

        utxos.sort_by(|a, b| a.value.cmp(&b.value));

        Ok(utxos)
    }

    async fn get_unspent_address<S>(&self, address: S) -> AnyhowResult<Vec<Utxo>>
    where
        S: AsRef<str>,
    {
        let scripthash = Address::from_str(address.as_ref())?.require_network(*self.network())?;

        self.get_unspent_scripthash(utils::address2scripthash(&scripthash)?)
            .await
    }

    async fn wait_util_utxo<S>(&self, address: S, satoshi: u64) -> AnyhowResult<Utxo>
    where
        S: AsRef<str>,
    {
        loop {
            for u in self.get_unspent_address(address.as_ref()).await? {
                if u.atomicals.is_empty() && u.value >= satoshi {
                    return Ok(u);
                }
            }

            tracing::info!("Waiting for UTXO ...");

            time::sleep(Duration::from_secs(5)).await;
        }
    }

    async fn broadcast<S>(&self, tx: S) -> AnyhowResult<serde_json::Value>
    where
        S: AsRef<str>,
    {
        self.post::<_, _, serde_json::Value>(
            self.uri_of("blockchain.transaction.broadcast"),
            Params::new([tx.as_ref()]),
        )
        .await
    }
}

impl<T> Api for T where T: Config + Http {}
