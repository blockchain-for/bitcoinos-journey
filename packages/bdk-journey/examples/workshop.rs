// use std::{collections::BTreeMap, fs::File, io::Read};

// use bdk::{
//     bitcoin::{
//         bip32::{self, ExtendedPrivKey},
//         secp256k1::{rand, rand::RngCore, Secp256k1},
//         Address, Network, OutPoint, ScriptBuf, Txid,
//     },
//     descriptor::IntoWalletDescriptor,
//     keys::IntoDescriptorKey,
//     wallet::{AddressIndex, Update},
//     KeychainKind, Wallet,
// };

// use bdk::descriptor;
// use bdk_esplora::{esplora_client, EsploraAsyncExt};
// use bdk_file_store::Store;

// use std::io::Write;
// use std::str::FromStr;

// const CONFIG_FILE: &str = "config.txt";
// const CHAIN_DATA_FILE: &str = "chain.dat";
// const DB_MAGIC: &[u8] = "TABCONF24".as_bytes();

// const STOP_GAP: usize = 50;
// const PARALLEL_REQUESTS: usize = 5;

// const SEND_AMOUNT: u64 = 5000;

// #[tokio::main]
// async fn main() -> Result<(), Box<dyn std::error::Error>> {
//     // Create and load or save new descriptors

//     let secp = Secp256k1::new();
//     let network = Network::Signet;

//     // get descriptors from config.txt, otherwise create it if file isn't exist
//     let (external_desc, internal_desc) = match File::open(CONFIG_FILE) {
//         Ok(mut file) => {
//             let mut config = String::new();
//             file.read_to_string(&mut config)?;
//             let descriptor_strs: [_; 2] = config
//                 .split('|')
//                 .map(|s| s.to_string())
//                 .collect::<Vec<_>>()
//                 .try_into()
//                 .unwrap();

//             let external_desc = descriptor_strs[0].into_wallet_descriptor(&secp, network)?;
//             let internal_desc = descriptor_strs[1].into_wallet_descriptor(&secp, network)?;
//             (external_desc, internal_desc)
//         }
//         Err(_) => {
//             // Create new descriptor and save
//             let mut seed = [0u8; 32];
//             rand::thread_rng().fill_bytes(&mut seed);

//             // path seed -> ExtendedPrivKey -> DerivationPath ->
//             let xprv = ExtendedPrivKey::new_master(network, &seed)?;
//             let bip86_external = bip32::DerivationPath::from_str("m/86'/1'/0'/0/0")?;
//             let bip86_internal = bip32::DerivationPath::from_str("m/86'/1'/0'/0/1")?;
//             let extendey_key = (xprv, bip86_external).into_descriptor_key()?;
//             let internal_key = (xprv, bip86_internal).into_descriptor_key()?;

//             let external_desc =
//                 descriptor!(tr(extendey_key))?.into_wallet_descriptor(&secp, network)?;
//             let internal_desc =
//                 descriptor!(tr(internal_key))?.into_wallet_descriptor(&secp, network)?;
//             let config = format!(
//                 "{}|{}",
//                 external_desc.0.to_string_with_secret(&internal_desc.1),
//                 internal_desc.0.to_string_with_secret(&internal_desc.1)
//             );

//             // Save descriptors string to file
//             let mut file = File::create(CONFIG_FILE)?;
//             file.write_all(config.as_bytes())?;

//             (external_desc, internal_desc)
//         }
//     };

//     println!(
//         "External descriptor: {}",
//         &external_desc.0.to_string_with_secret(&external_desc.1)
//     );
//     println!(
//         "Internal descriptor: {}\n",
//         &internal_desc.0.to_string_with_secret(&internal_desc.1)
//     );

//     // Create a wallet and get a new address
//     type ChangeSet = Vec<String>;
//     let db = Store::<ChangeSet>::new_from_path(DB_MAGIC, CHAIN_DATA_FILE)?;

//     let wallet = Wallet::new(external_desc, Some(internal_desc), network, db.into())?;

//     let address = wallet.get_address(AddressIndex::New)?;
//     let balance = wallet.get_balance()?;

//     let client = esplora_client::Builder::new("http://signet.bitcoindevkit.net").build_async()?;
//     let prev_tip = wallet.latest_checkpoint();

//     if prompt("Scan wallet") {
//         let keychain_spks = wallet
//             .spks_of_all_keychains()
//             .into_iter()
//             .map(|(keychain, iter)| {
//                 let mut first = true;
//                 let spks_iter = iter.inspect(move |(i, _)| {
//                     if first {
//                         eprint!(
//                             "\nscanning: {}",
//                             match keychain {
//                                 KeychainKind::External => "External",
//                                 KeychainKind::Internal => "Internal",
//                             }
//                         );

//                         first = false;
//                     }

//                     eprint!("{}", i);
//                     // Flush early to ensure print at every iteration
//                     let _ = std::io::stderr().flush();
//                 });
//                 (keychain, spks_iter)
//             })
//             .collect::<BTreeMap<_, _>>();

//         // The client scan keychain spks for transaction histories, stopping after `stop_gap` is reached.
//         // It returns a `TxGraph` update (`graph_update`) and a structure that represents the
//         // last active  spk derivation indices of keychains (`keychain_indices_update`)
//         let (graph_update, last_active_indices) = client
//             .scan_txs_with_keychains(
//                 keychain_spks,
//                 core::iter::empty(),
//                 core::iter::empty(),
//                 STOP_GAP,
//                 PARALLEL_REQUESTS,
//             )
//             .await?;
//         println!();

//         let missing_heights = graph_update.missing_heights(wallet.local_chain());

//         let chain_update = client
//             .update_local_chain(prev_tip.clone(), missing_heights)
//             .await?;
//         let update = Update {
//             last_active_indices,
//             graph: graph_update,
//             chain: Some(chain_update),
//         };

//         wallet.apply_update(update)?;

//         wallet.commit()?;
//         println!("Scan completed!");

//         let balance = wallet.get_balance()?;
//         println!("Wallet balance after scanning: confirmed {} sats, trusted_pending {} sats, untrusted pending {} sats",
//         balance.confirmed, balance.trusted_pending, balance.untrusted_pending);
//     } else {
//         // Syncing: Only check for specified spks, utxos and txid to update their confirmation status or fetch missing transaction Spks.
//         // outpoints and txids we want updates on will be accumulated here

//         let mut spks: Box<Vec<ScriptBuf>> = Box::new(vec![]);
//         let mut outpoints: Box<dyn Iterator<Item = OutPoint> + Send> =
//             Box::new(core::iter::empty());
//         let mut txids: Box<dyn Iterator<Item = Txid> + Send> = Box::new(core::iter::empty());

//         // Sync all SPKs
//         if prompt("Sync all SPKs") {
//             let all_spks: Vec<ScriptBuf> = wallet
//                 .spk_index()
//                 .all_spks()
//                 .into_iter()
//                 .map(|((keychain, index), script)| {
//                     eprintln!(
//                         "Checking if keychain: {}, index: {}, address: {} has been used",
//                         match keychain {
//                             KeychainKind::External => "External",
//                             KeychainKind::Internal => "Internal",
//                         },
//                         index,
//                         Address::from_script(script.as_script(), network).unwrap(),
//                     );

//                     let _ = std::io::stderr().flush();
//                     (*script).clone()
//                 })
//                 .collect();

//             spks = Box::new(all_spks);
//         }
//         // Sync only unused SPKs
//         else if prompt("Sync only unused SPKs") {
//             // TODO add Wallet::unused_spks() function, gives all unused tracked spks
//             let unused_spks: Vec<ScriptBuf> = wallet
//                 .spk_index()
//                 .unused_spks(..)
//                 .into_iter()
//                 .map(|((keychain, index), script)| {
//                     eprintln!(
//                         "Checking if keychain: {}, index: {}, address: {} has been used",
//                         match keychain {
//                             KeychainKind::External => "External",
//                             KeychainKind::Internal => "Internal",
//                         },
//                         index,
//                         Address::from_script(script, network).unwrap(),
//                     );
//                     // Flush early to ensure we print at every iteration.
//                     let _ = std::io::stderr().flush();
//                     ScriptBuf::from(script)
//                 })
//                 .collect();
//             spks = Box::new(unused_spks);
//         }

//         // Sync UTXOs
//         if prompt("Sync UTXOs") {
//             // We want to search for whether the UTXO is spent, and spent by which
//             // transaction. We provide the outpoint of the UTXO to
//             // `EsploraExt::update_tx_graph_without_keychain`.
//             let utxo_outpoints = wallet
//                 .list_unspent()
//                 .inspect(|utxo| {
//                     eprintln!(
//                         "Checking if outpoint {} (value: {}) has been spent",
//                         utxo.outpoint, utxo.txout.value
//                     );
//                     // Flush early to ensure we print at every iteration.
//                     let _ = io::stderr().flush();
//                 })
//                 .map(|utxo| utxo.outpoint);
//             outpoints = Box::new(utxo_outpoints);
//         };

//         // Sync unconfirmed TX
//         if prompt("Sync unconfirmed TX") {
//             // We want to search for whether the unconfirmed transaction is now confirmed.
//             // We provide the unconfirmed txids to
//             // `EsploraExt::update_tx_graph_without_keychain`.
//             let unconfirmed_txids = wallet
//                 .transactions()
//                 .filter(|canonical_tx| !canonical_tx.chain_position.is_confirmed())
//                 .map(|canonical_tx| canonical_tx.tx_node.txid)
//                 .inspect(|txid| {
//                     eprintln!("Checking if {} is confirmed yet", txid);
//                     // Flush early to ensure we print at every iteration.
//                     let _ = std::io::stderr().flush();
//                 });
//             txids = Box::new(unconfirmed_txids);
//         }

//         let graph_update = client
//             .scan_txs(spks.into_iter(), txids, outpoints, PARALLEL_REQUESTS)
//             .await?;

//         let missing_heights = graph_update.missing_heights(wallet.local_chain());
//         let chain_update = client.update_local_chain(prev_tip, missing_heights).await?;

//         let update = Update {
//             // no update to active indices
//             last_active_indices: BTreeMap::new(),
//             graph: graph_update,
//             chain: Some(chain_update),
//         };
//         wallet.apply_update(update)?;
//         wallet.commit()?;
//         println!("Sync completed.");

//         let balance = wallet.get_balance();
//         println!("Wallet balance after syncing: confirmed {} sats, trusted_pending {} sats, untrusted pending {} sats",
//                  balance.confirmed, balance.trusted_pending, balance.untrusted_pending);
//     }

//     // Check balance and request deposit if required
//     if balance.total() < SEND_AMOUNT {
//         println!(
//             "Please send at least {} sats to {} using: https://signetfaucet.com/",
//             SEND_AMOUNT, address.address
//         );
//         std::process::exit(0);
//     }

//     // Create TX to return sats to signet faucet https://signetfaucet.com/
//     let faucet_address = Address::from_str("tb1qg3lau83hm9e9tdvzr5k7aqtw3uv0dwkfct4xdn")?
//         .require_network(network)?;

//     let mut tx_builder = wallet.build_tx();
//     tx_builder
//         .add_recipient(faucet_address.script_pubkey(), SEND_AMOUNT)
//         // .drain_to(faucet_address.script_pubkey())
//         // .drain_wallet()
//         .fee_rate(FeeRate::from_sat_per_vb(2.1))
//         .enable_rbf();

//     let mut psbt = tx_builder.finish()?;
//     let finalized = wallet.sign(&mut psbt, SignOptions::default())?;
//     assert!(finalized);

//     let tx = psbt.extract_tx();
//     let (sent, received) = wallet.sent_and_received(&tx);
//     let fee = wallet.calculate_fee(&tx).expect("fee");
//     let fee_rate = wallet
//         .calculate_fee_rate(&tx)
//         .expect("fee rate")
//         .as_sat_per_vb();
//     println!(
//         "Created tx sending {} sats to {}",
//         sent - received - fee,
//         faucet_address
//     );
//     println!(
//         "Fee is {} sats, fee rate is {:.2} sats/vbyte",
//         fee, fee_rate
//     );

//     if prompt("Broadcast") {
//         client.broadcast(&tx).await?;
//         println!(
//             "Tx broadcast! https://mempool.space/signet/tx/{}",
//             tx.txid()
//         );
//     }

//     Ok(())
// }

// pub fn prompt(question: &str) -> bool {
//     print!("{}? (Y/N)", question);
//     std::io::stdout().flush().expect("stdout flush");
//     let mut answer = String::new();

//     std::io::stdin().read_line(&mut answer).expect("answer");
//     answer.trim().to_ascii_lowercase() == "y"
// }

fn main() {
    
}