use std::{fs::File, io::Read};

use bdk::{
    bitcoin::{
        bip32::{self, ExtendedPrivKey},
        secp256k1::{rand, rand::RngCore, Secp256k1},
        Network,
    },
    descriptor::IntoWalletDescriptor,
    keys::IntoDescriptorKey,
    // wallet::AddressIndex,
    // Wallet,
};

use bdk::descriptor;
// use bdk_esplora::esplora_client;
// use bdk_file_store::Store;

use std::io::Write;
use std::str::FromStr;

const CONFIG_FILE: &str = "config.txt";
// const CHAIN_DATA_FILE: &str = "chain.dat";
// const DB_MAGIC: &[u8] = "TABCONF24".as_bytes();

// const STOP_GAP: usize = 50;
// const PARALLEL_REQUESTS: usize = 5;

// const SEND_AMOUNT: u64 = 5000;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create and load or save new descriptors

    let secp = Secp256k1::new();
    let network = Network::Signet;

    // get descriptors from config.txt, otherwise create it if file isn't exist
    let (external_desc, internal_desc) = match File::open(CONFIG_FILE) {
        Ok(mut file) => {
            let mut config = String::new();
            file.read_to_string(&mut config)?;
            let descriptor_strs: [_; 2] = config
                .split('|')
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
            let config = format!(
                "{}|{}",
                external_desc.0.to_string_with_secret(&internal_desc.1),
                internal_desc.0.to_string_with_secret(&internal_desc.1)
            );

            // Save descriptors string to file
            let mut file = File::create(CONFIG_FILE)?;
            file.write_all(config.as_bytes())?;

            (external_desc, internal_desc)
        }
    };

    println!(
        "External descriptor: {}",
        &external_desc.0.to_string_with_secret(&external_desc.1)
    );
    println!(
        "Internal descriptor: {}\n",
        &internal_desc.0.to_string_with_secret(&internal_desc.1)
    );

    // Create a wallet and get a new address
    // type ChangeSet = Vec<String>;
    // let db = Store::<ChangeSet>::new_from_path(DB_MAGIC, CHAIN_DATA_FILE)?;

    // let wallet = Wallet::new(external_desc, Some(internal_desc), network, db.into())?;

    // let address = wallet.get_address(AddressIndex::New)?;
    // let balance = wallet.get_balance()?;

    // let client = esplora_client::Builder::new("http://signet.bitcoindevkit.net").build_async()?;
    // let priv_tip = wallet.latest_checkpoint();

    Ok(())
}

pub fn prompt(question: &str) -> bool {
    print!("{}? (Y/N)", question);
    std::io::stdout().flush().expect("stdout flush");
    let mut answer = String::new();

    std::io::stdin().read_line(&mut answer).expect("answer");
    answer.trim().to_ascii_lowercase() == "y"
}
