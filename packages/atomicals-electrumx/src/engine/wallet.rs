use std::str::FromStr;

use bitcoin::{secp256k1::Keypair, Address, Network, XOnlyPublicKey};

use crate::{utils, AnyhowResult};

use crate::wallet::Wallet;

#[derive(Debug, Clone)]
pub(super) struct EngineWallet {
    stash: Key,
    funding: Key,
}

impl EngineWallet {
    pub(super) fn from_raw_wallet(wallet: Wallet, network: Network) -> AnyhowResult<Self> {
        let stash_pair = utils::keypair_from_wif(&wallet.stash.key.wif)?;
        let funding_pair = utils::keypair_from_wif(&wallet.funding.wif)?;

        Ok(Self {
            stash: Key {
                pair: stash_pair,
                x_only_public_key: stash_pair.x_only_public_key().0,
                address: Address::from_str(&wallet.stash.key.address)?.require_network(network)?,
            },
            funding: Key {
                pair: funding_pair,
                x_only_public_key: funding_pair.x_only_public_key().0,
                address: Address::from_str(&wallet.funding.address)?.require_network(network)?,
            },
        })
    }
}

#[derive(Debug, Clone)]
pub(super) struct Key {
    pair: Keypair,
    x_only_public_key: XOnlyPublicKey,
    address: Address,
}

#[derive(Debug, Clone)]
pub(super) struct KeyAlias {
    alias: String,
    key: String,
}
