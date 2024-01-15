pub mod http;

use std::time::Duration;

use bitcoin::network::Network;
use reqwest::{Client as ReqwestClient, ClientBuilder};

pub use http::*;
use serde::{de::DeserializeOwned, Serialize};

use crate::AnyhowResult;

#[derive(Debug)]
pub struct ElectrumX {
    pub client: ReqwestClient,
    pub network: Network,
    pub base_uri: String,
}

impl Config for ElectrumX {
    fn network(&self) -> &Network {
        &self.network
    }

    fn base_uri(&self) -> &str {
        &self.base_uri
    }
}

impl Http for ElectrumX {
    async fn post<U, P, R>(&self, uri: U, params: P) -> AnyhowResult<R>
    where
        U: AsRef<str>,
        P: Serialize,
        R: DeserializeOwned,
    {
        let resp = self
            .client
            .post(uri.as_ref())
            .json(&params)
            .send()
            .await?
            .text()
            .await?;

        tracing::info!("{:?}", resp);

        Ok(serde_json::from_str(&resp)?)
    }
}

impl ElectrumX {
    pub fn new(
        network: Network,
        base_uri: impl Into<String>,
        timeout_secs: u64,
    ) -> AnyhowResult<Self> {
        Ok(Self {
            client: ClientBuilder::new()
                .timeout(Duration::from_secs(timeout_secs))
                .build()?,
            network,
            base_uri: base_uri.into(),
        })
    }
}
