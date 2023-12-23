use std::str::FromStr;

use bdk::{
    bitcoin::{Address, Network},
    blockchain::{Blockchain, ElectrumBlockchain},
    database::MemoryDatabase,
    electrum_client::Client,
    wallet::AddressIndex,
    SignOptions, SyncOptions, Wallet,
};

/// Generate new address from descriptors
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let external_desc = "wpkh(tprv8ZgxMBicQKsPdy6LMhUtFHAgpocR8GC6QmwMSFpZs7h6Eziw3SpThFfczTDh5rW2krkqffa11UpX3XkeTTB2FvzZKWXqPY54Y6Rq4AQ5R8L/84'/0'/0'/0/*)";
    let internal_desc = "wpkh(tprv8ZgxMBicQKsPdy6LMhUtFHAgpocR8GC6QmwMSFpZs7h6Eziw3SpThFfczTDh5rW2krkqffa11UpX3XkeTTB2FvzZKWXqPY54Y6Rq4AQ5R8L/84'/0'/0'/1/*)";

    let wallet = Wallet::new(
        external_desc,
        Some(internal_desc),
        Network::Testnet,
        MemoryDatabase::default(),
    )
    .expect("Can't create wallet!");

    let address = wallet
        .get_address(AddressIndex::New)
        .expect("Can't create new address");

    println!("Generated address: {:?}", address.address);

    let client = Client::new("ssl://electrum.blockstream.info:60002")?;
    let blockchain = ElectrumBlockchain::from(client);

    wallet.sync(&blockchain, SyncOptions::default())?;

    let balance = wallet.get_balance()?;
    println!("Wallet Balance is in SATs: {}", balance);

    let faucet_address = Address::from_str("mkHS9ne12qx9pS9VojpwU5xtRd4T7X7ZUt")?;

    let mut tx_builder = wallet.build_tx();

    tx_builder
        .add_recipient(
            faucet_address.payload.script_pubkey(),
            (balance.trusted_pending + balance.confirmed) / 2,
        )
        .enable_rbf();

    let (mut psbt, tx_details) = tx_builder.finish()?;

    println!("Transaction details: {:#?}", tx_details);

    let finalized = wallet.sign(&mut psbt, SignOptions::default())?;

    assert!(finalized, "Tx has not been finalized");
    println!("Transaction Signed: {}", finalized);

    let raw_transaction = psbt.extract_tx();
    let txid = raw_transaction.txid();
    blockchain.broadcast(&raw_transaction)?;
    println!(
        "Transaction sent: TXID: {txid}.\nExplorer URL: https://blockstream.info/testnet/tx/{txid}",
        txid = txid,
    );

    Ok(())
}
