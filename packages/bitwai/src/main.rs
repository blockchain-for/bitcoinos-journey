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
};

fn main() {
    let (receive, change) = get_descriptors();
    println!("recv: {:#?}, \nchange: {:#?}", receive, change);
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
