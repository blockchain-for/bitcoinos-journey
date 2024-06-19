use std::{
    fs::File,
    io::{self, prelude::*},
    path::{Path, PathBuf},
};

use rand_chacha::ChaCha20Rng;
use rand_core::{RngCore, SeedableRng};

use sha2::{Digest, Sha256, Sha512};

/// This struct takes a constant `N` as a generic
/// Enabling one to specify a variable length for the bytes generated
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Entropy<const N: usize>([u8; N]);

impl<const N: usize> Entropy<N> {
    pub fn generate() -> Self {
        // Instantiate our cryptographically secure random byte generation algorithm
        let mut rng = ChaCha20Rng::from_entropy();

        // Create a zero filled buffer to hold our bytes
        let mut buffer = [0u8; N];

        // Fill the buffer with random bytes
        rng.fill_bytes(&mut buffer);

        // Return the buffer
        Self(buffer)
    }
}

#[derive(Debug, Default)]
pub struct Bip39Generator {
    // This holds all our indexes that we will use to fetch our words from the word list
    // with eache index corresponding to an index from our word word list contained in a Vec<word>
    mnemonic_index: Vec<u16>,
    // This field holds the random bytes with our checksum bytes appended to the end
    appended: Vec<u8>,
    // This contains a path to our word list file
    path: PathBuf,
}

impl Bip39Generator {
    // This method takes an argument `path_to_wordlist` which is a path to the wordlist
    // where the path is anything that implements the trait `AsRef<Path>` meaning we pass any data
    // type as convert it to a path using the `.as_ref()` method as long as that
    // data type implements the `AsRef<Path>` trait.
    pub fn new(path_to_wordlist: impl AsRef<Path>) -> Self {
        Self {
            // Convert `path_to_wordlist` argument to a path using `.as_ref()` method and convert it
            // to a `std::path::PathBuf` using the `.to_path_buf()`
            path: path_to_wordlist.as_ref().to_path_buf(),
            // All other fields are default
            ..Default::default()
        }
    }

    pub fn load_wordlist(&mut self) -> io::Result<Vec<String>> {
        let file = File::open(&self.path)?;

        let reader: io::BufReader<File> = io::BufReader::new(file);

        reader.lines().collect()
    }

    pub fn generate_checksum<const N: usize>(&mut self, entropy: [u8; N]) -> &mut Self {
        // BIP39 spec requires a seed to generated
        // using a SHA256 Psuedo Random Function (PRF)
        // so we instantiate a SHA256 hashing function
        let mut hasher = Sha256::new();

        // We now pass our random bytes into the PRF
        hasher.update(entropy.as_slice());

        let entropy_hash = hasher.finalize();

        // Since we get a 32 bytes value we multiply it by `8` to get number of bits
        // since 1 byte = 8 bits
        let bits_of_entroy = entropy.len() * 8;

        let bits_of_checksum = bits_of_entroy / 32;

        let significant = entropy_hash[0] >> bits_of_checksum;

        let mut appended = entropy.to_vec();

        appended.push(significant);

        self.appended = appended;

        self
    }
}
