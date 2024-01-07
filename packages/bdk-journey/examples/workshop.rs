use std::{fs::File, io::Read};

use bdk::{
    bitcoin::{
        bip32::{self, ExtendedPrivKey},
        secp256k1::{rand, rand::RngCore, Secp256k1},
        Network,
    },
    descriptor::IntoWalletDescriptor,
    keys::IntoDescriptorKey,
};

use bdk::descriptor;

use std::str::FromStr;

const CONFIG_FILE: &str = "config.txt";
const CHAIN_DATA_FILE: &str = "chain.dat";
const DB_MAGIC: &[u8] = "TABCONF24".as_bytes();

const STOP_GAP: usize = 50;
const PARALLEL_REQUESTS: usize = 5;

const SEND_AMOUNT: u64 = 5000;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create and load or save new descriptors

    let secp = Secp256k1::new();
    let network = Network::Signet;

    // get descriptors from config.txt, otherwise create it if file isn't exist
    let descriptors = match File::open(CONFIG_FILE) {
        Ok(mut file) => {
            let mut config = String::new();
            file.read_to_string(&mut config)?;
            let descriptor_strs: [_; 2] = config
                .split("|")
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
                .try_into()
                .unwrap();

            let external_desc = descriptor_strs[0].into_wallet_descriptor(&secp, network)?;
            let internal_desc = descriptor_strs[1].into_wallet_descriptor(&secp, network)?;
            (external_desc, internal_desc)
        }
        Err(_) => {
            // Create new descriptor and save
            let mut seed = [0u8; 32];
            rand::thread_rng().fill_bytes(&mut seed);

            // path seed -> ExtendedPrivKey -> DerivationPath ->
            let xprv = ExtendedPrivKey::new_master(network, &seed)?;
            let bip86_external = bip32::DerivationPath::from_str("m/86'/1'/0'/0/0")?;
            let bip86_internal = bip32::DerivationPath::from_str("m/86'/1'/0'/0/1")?;
            let extendey_key = (xprv, bip86_external).into_descriptor_key()?;
            let internal_key = (xprv, bip86_internal).into_descriptor_key()?;

            let external_desc =
                descriptor!(tr(extendey_key))?.into_wallet_descriptor(&secp, network)?;
            let internal_desc =
                descriptor!(tr(internal_key))?.into_wallet_descriptor(&secp, network)?;

            todo!()
        }
    };

    Ok(())
}
