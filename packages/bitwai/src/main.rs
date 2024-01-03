use std::str::FromStr;

use bdk::{
    bitcoin::{
        secp256k1::{Secp256k1, SecretKey},
        util::bip32::{DerivationPath, KeySource},
        Amount, Network,
    },
    bitcoincore_rpc::{Auth as rpc_auth, Client, RpcApi},
    blockchain::{
        rpc::{wallet_name_from_descriptor, Auth, RpcBlockchain, RpcConfig},
        ConfigurableBlockchain, NoopProgress,
    },
    keys::{
        bip39::{Language, Mnemonic, MnemonicType},
        DerivableKey,
        DescriptorKey::{self, Secret},
        ExtendedKey, GeneratableKey, GeneratedKey,
    },
    miniscript::Segwitv0,
    sled,
    wallet::{tx_builder, AddressIndex},
    SignOptions, TxBuilder, Wallet,
};

fn main() {
    // Create a RPC client
    let rpc_auth = rpc_auth::UserPass("user".to_string(), "password".to_string());

    let rpc = Client::new("http://127.0.0.1:18443/wallet/bitwai".to_string(), rpc_auth).unwrap();
    println!("bitcoin info: {:#?}", rpc.get_blockchain_info());

    // create bitwai wallet
    rpc.create_wallet("bitwai", None, None, None, None).unwrap();
    // get wwallet address
    let core_address = rpc.get_new_address(None, None).unwrap();
    println!("Core address: {:#?}", core_address);

    // Generate 101 blocks and use the above address as coinbase
    rpc.generate_to_address(101, &core_address).unwrap();

    let core_balance = rpc.get_balance(None, None).unwrap();
    // Show balance
    println!("Balance: {:#?}", core_balance);

    let (recv_desc, change_desc) = get_descriptors();
    println!("recv: {:#?}, \nchange: {:#?}", recv_desc, change_desc);

    let wallet_name = wallet_name_from_descriptor(
        &recv_desc,
        Some(&change_desc),
        Network::Regtest,
        &Secp256k1::new(),
    )
    .unwrap();

    // Create the datadir to store the wallet data
    let mut datadir = dirs_next::home_dir().unwrap();
    datadir.push(".bitwai");
    let database = sled::open(datadir).unwrap();
    let db_tree = database.open_tree(wallet_name.clone()).unwrap();

    // Set RPC username
    let rpc_url = "http://127.0.0.1:18443".to_string();
    let auth = Auth::UserPass {
        username: "user".to_string(),
        password: "password".to_string(),
    };
    let rpc_config = RpcConfig {
        url: rpc_url,
        auth,
        network: Network::Regtest,
        wallet_name,
        skip_blocks: None,
    };

    // Create blockchain backend with config
    let blockchain = RpcBlockchain::from_config(&rpc_config).unwrap();

    let wallet = Wallet::new(
        &recv_desc,
        Some(&change_desc),
        Network::Regtest,
        db_tree,
        blockchain,
    )
    .unwrap();

    wallet.sync(NoopProgress, None).unwrap();

    // Fetch a fresh address to receive coins
    let address = wallet.get_address(AddressIndex::New).unwrap();

    println!("bdk client address: {:#?}", address);

    // Send 10 BTC from Core to BDK client
    rpc.send_to_address(
        &address,
        Amount::from_btc(10.0).unwrap(),
        None,
        None,
        None,
        None,
        None,
        None,
    )
    .unwrap();

    // Confirm transaction by generate some blocks
    rpc.generate_to_address(1, &core_address).unwrap();

    // Sync the BDK client wallet
    wallet.sync(NoopProgress, None).unwrap();

    // Create a transaction builder using wallet
    let mut tx_builder = wallet.build_tx();

    // Set recipient of the tx
    tx_builder.set_recipients(vec![(core_address.script_pubkey(), 500_000_000)]);

    // Finalise the tx and extract PSBT
    let (mut psbt, _) = tx_builder.finish().unwrap();

    // Sign the PSBT
    let sign_option = SignOptions {
        assume_height: None,
        ..Default::default()
    };
    wallet.sign(&mut psbt, sign_option).unwrap();

    // Extract the final transaction
    let tx = psbt.extract_tx();

    wallet.broadcast(tx).unwrap();

    // Confirm transaction by generating some blocks
    rpc.generate_to_address(1, &core_address).unwrap();

    // Sync the BDK client wallet
    wallet.sync(NoopProgress, None).unwrap();

    // Fetch and display wallet balances
    let core_balance = rpc.get_balance(None, None).unwrap();
    let bdk_balance = Amount::from_sat(wallet.get_balance().unwrap());
    println!("Core wallet Balance: {:#?}", core_balance);
    println!("BDK wallet Balance: {:#?}", bdk_balance);
}

// Generate fresh desciptor strings and return them via (receive, change) tuple
fn get_descriptors() -> (String, String) {
    // Create a new secret context
    let secp = Secp256k1::new();

    let password = Some("12345678".to_string());

    // Generate a mnemonic, and from there a private key
    let mnemonic: GeneratedKey<_, Segwitv0> =
        Mnemonic::generate((MnemonicType::Words12, Language::English)).unwrap();

    let mnemonic = mnemonic.into_key();

    let xkey: ExtendedKey = (mnemonic, password).into_extended_key().unwrap();

    let xprv = xkey.into_xprv(Network::Regtest).unwrap();

    // Create derived private from the above master privkey
    // Using the following derivation paths for receive and change keys
    // receive: "m/84h/1h/0h/0"
    // change:  "m/84h/1h/0h/1"
    let receive_derivation_path = "m/84h/1h/0h/0";
    let change_derivation_path = "m/84h/1h/0h/1";

    let mut keys = vec![];

    for path in [receive_derivation_path, change_derivation_path] {
        let deriv_path: DerivationPath = DerivationPath::from_str(path).unwrap();
        let derived_xprv = &xprv.derive_priv(&secp, &deriv_path).unwrap();
        let origin = (xprv.fingerprint(&secp), deriv_path);

        let derived_xprv_desc_key: DescriptorKey<Segwitv0> = derived_xprv
            .into_descriptor_key(Some(origin), DerivationPath::default())
            .unwrap();

        // Wrap the derived key with the wpkh() string to produce a descriptor string
        if let Secret(key, _, _) = derived_xprv_desc_key {
            let desc = format!("wpkh({})", key);

            keys.push(desc);
        }
    }

    (keys[0].clone(), keys[1].clone())
}
