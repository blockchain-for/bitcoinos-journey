use std::{
    fs::File,
    io::{self, prelude::*},
    path::{Path, PathBuf},
};

use pbkdf2::pbkdf2_hmac;
use rand_chacha::ChaCha20Rng;
use rand_core::{RngCore, SeedableRng};

use sha2::{Digest, Sha256, Sha512};

use crate::{ITERATION_COUNT, SALT_PREFIX};

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

    pub fn compute(&mut self) -> &mut Self {
        let mut bits = vec![];

        for &byte in self.appended.iter() {
            for i in (0..8).rev() {
                bits.push((byte >> i) & 1u8 == 1);
            }
        }

        for chunk in bits.chunks(11) {
            if chunk.len() == 11 {
                let mut value: u16 = 0;

                for (i, &bit) in chunk.iter().enumerate() {
                    if bit {
                        value |= 1u16 << (10 - i);
                    }
                }
                self.mnemonic_index.push(value);
            }
        }

        self
    }

    pub fn mnemonic<const N: usize>(&mut self) -> io::Result<String> {
        let entropy = Entropy::<N>::generate();

        self.generate_checksum(entropy.0);

        self.compute();

        let wordlist = self.load_wordlist()?;

        let mnemonic = self
            .mnemonic_index
            .iter()
            .enumerate()
            .map(|(index, line_number)| {
                let word = wordlist[*line_number as usize].clone() + " ";

                let index = index + 1;

                // Check if we have our index is less than 10 so we add a padding to making printing
                let indexed = if index < 10 {
                    String::new() + " " + index.to_string().as_str()
                } else {
                    index.to_string()
                };

                // Print our index and each word.
                // This will show the user the words in each line but with a number. e.g.
                //  9. foo
                // 10. bar
                println!("{}. {}", indexed, &word);

                word
            })
            .collect::<String>();

        Ok(mnemonic.trim().to_owned())
    }

    fn seed(mnemonic: &str, passphrase: Option<&str>) -> io::Result<Vec<u8>> {
        let salt = if let Some(passphrase_required) = passphrase {
            String::new() + SALT_PREFIX + passphrase_required
        } else {
            String::from(SALT_PREFIX)
        };

        let mut wallet_seed = [0u8; 64]; // 512 bits = 64 bytes

        pbkdf2_hmac::<Sha512>(
            mnemonic.as_bytes(),
            salt.as_bytes(),
            ITERATION_COUNT,
            &mut wallet_seed,
        );

        Ok(wallet_seed.to_vec())
    }

    pub fn insecure_seed(mnemonic: &str) -> io::Result<Vec<u8>> {
        Self::seed(mnemonic, None)
    }

    pub fn secure_seed(mnemonic: &str, passphrase: &str) -> io::Result<Vec<u8>> {
        Self::seed(mnemonic, Some(passphrase))
    }
}
