// use pbkdf2::pbkdf2_hmac;
// use rand_chacha::ChaCha20Rng;
// use rand_core::{RngCore, SeedableRng};
// use sha2::{Digest, Sha256, Sha512};

// use std::{
//     fs::File,
//     io::{self, prelude::*},
//     path::{Path, PathBuf},
// };

use std::env;

use bip39_simple::domain::Bip39Generator;

fn main() {
    let current_path = env::current_dir().unwrap();
    println!("Current path: {:?}", current_path.display());

    let crate_path = module_path!();
    println!("Crate path: {:?}", crate_path);
    let full_path = format!(
        "{}/{}/english.txt",
        current_path.to_str().unwrap(),
        crate_path
    );
    println!("Full path: {:?}", full_path);
    let mut generator = Bip39Generator::new(full_path);

    let mnemonic = generator.mnemonic::<16>().unwrap();

    println!("Your mnemonic is: {}", &mnemonic);

    let insecure_seed = Bip39Generator::insecure_seed(&mnemonic);

    let secure_seed = Bip39Generator::secure_seed(&mnemonic, "BitcoinOS");

    assert!(insecure_seed.is_ok());
    assert!(secure_seed.is_ok());
}
