use std::{
    collections::HashMap,
    fs::{self, File},
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::AnyhowResult;

#[derive(Debug, Clone)]
pub struct Wallet {
    pub path: PathBuf,
    pub stash: KeyAlias,
    pub funding: Key,
}

impl Wallet {
    pub fn load<P>(path: P) -> AnyhowResult<Self>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let wallet = serde_json::from_reader::<_, WalletJson>(File::open(path)?)?;

        Ok(Self {
            path: path.to_path_buf(),
            stash: wallet
                .imported
                .get("stash")
                .map(|k| KeyAlias {
                    alias: "stash".into(),
                    key: k.to_owned(),
                })
                .unwrap_or(KeyAlias {
                    alias: "primary".into(),
                    key: wallet.primary.to_owned(),
                }),
            funding: wallet.funding,
        })
    }

    pub fn load_wallets<P>(path: P) -> Vec<Wallet>
    where
        P: AsRef<Path>,
    {
        fs::read_dir(path)
            .ok()
            .map(|rd| {
                rd.filter_map(|rde| {
                    rde.ok().and_then(|de| {
                        let pathbuf = de.path();

                        if pathbuf.extension().map(|ex| ex == "json") == Some(true) {
                            Self::load(&pathbuf)
                                .map(|wallet| {
                                    tracing::info!("Loaded wallet: {}", pathbuf.display());
                                    wallet
                                })
                                .map_err(|e| {
                                    tracing::error!(
                                        "Failed to load wallet from: {},error: {}",
                                        pathbuf.display(),
                                        e
                                    );

                                    e
                                })
                                .ok()
                        } else {
                            None
                        }
                    })
                })
                .collect()
            })
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Key {
    pub address: String,
    #[serde(rename = "WIF")]
    pub wif: String,
}

#[derive(Debug, Clone)]
pub struct KeyAlias {
    pub alias: String,
    pub key: Key,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WalletJson {
    pub primary: Key,
    pub funding: Key,
    pub imported: HashMap<String, Key>,
}
