use bdk::{bitcoin::Network, database::SqliteDatabase, Wallet, wallet::AddressIndex, blockchain::ElectrumBlockchain, electrum_client::Client, SyncOptions};

fn main() -> anyhow::Result<()> {

    dotenv::dotenv().unwrap();

    let descriptor = std::env::var("WALLET__DESCRIPTOR").unwrap();

    // println!("{}", descriptor);

    let wallet = Wallet::new(
        &descriptor,
        None,
        Network::Testnet,
        SqliteDatabase::new("bitwai.db"),
    )?;
    
    let address = wallet
        .get_address(AddressIndex::New)
        .expect("Can't create new address");
    println!("Address: {}", address);

    let blockchain = ElectrumBlockchain::from(Client::new("ssl://electrum.blockstream.info:60002")?);
    let balance = wallet.get_balance()?;
    println!("Balance: {}", balance);

    wallet.sync(&blockchain, SyncOptions::default())?;

    Ok(())
}