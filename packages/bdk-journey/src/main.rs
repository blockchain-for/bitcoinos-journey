use bdk::bitcoin::secp256k1::Secp256k1;
use bdk::bitcoin::Network;
use bdk::database::MemoryDatabase;
use bdk::keys::bip39::{Language, Mnemonic, WordCount};
use bdk::keys::DerivableKey;
use bdk::keys::ExtendedKey;
use bdk::keys::GeneratableKey;
use bdk::keys::GeneratedKey;
use bdk::miniscript;
use bdk::template::Bip84;
use bdk::Wallet;

fn main() {
    println!("Hello, BDK developer!");

    let network = Network::Regtest;

    // Generate fresh mnemonic
    let mnemonic: GeneratedKey<_, miniscript::Segwitv0> =
        Mnemonic::generate((WordCount::Words12, Language::English)).unwrap();

    // Convert mnemonic to string
    let mnemonic_words = mnemonic.to_string();
    println!("Mnemonic words: {:?}", mnemonic_words);

    // Parse mnemonic
    let mnemonic = Mnemonic::parse(&mnemonic_words).unwrap();
    println!("Mnemonic: {:?}", mnemonic);

    let mnemonic_clone = mnemonic.clone();

    // Generate a extended key
    let xkey: ExtendedKey = mnemonic.into_extended_key().unwrap();
    let xkey2: ExtendedKey = mnemonic_clone.into_extended_key().unwrap();

    // Get xprv from the extended key
    let xprv = xkey.into_xprv(network).unwrap();
    println!("xprv: {:?}", xprv.to_string());

    println!("private key: {:?}", xprv.private_key.display_secret());

    let xpub = xkey2.into_xpub(network, &Secp256k1::default());
    println!("xpub: {:?}", xpub.to_string());

    // Create a BDK wallet structure use BIP 84 descriptor ("m/84h/1h/0h/0" and "m/84h/1h/0h/1")
    let wallet = Wallet::new(
        Bip84(xprv, bdk::KeychainKind::External),
        Some(Bip84(xprv, bdk::KeychainKind::Internal)),
        network,
        MemoryDatabase::default(),
    )
    .unwrap();

    println!(
        "mnemonic: {}\n\nrecv desc (pub key): {:#?}\n\nchng desc (pub key): {:#?}",
        mnemonic_words,
        wallet
            .get_descriptor_for_keychain(bdk::KeychainKind::External)
            .to_string(),
        wallet
            .get_descriptor_for_keychain(bdk::KeychainKind::Internal)
            .to_string(),
    );
}
