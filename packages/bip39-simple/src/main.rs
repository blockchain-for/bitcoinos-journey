use pbkdf2::pbkdf2_hmac;
use rand_chacha::ChaCha20Rng;
use rand_core::{RngCore, SeedableRng};
use sha2::{Digest, Sha256, Sha512};

use std::{
    fs::File,
    io::{self, prelude::*},
    path::{Path, PathBuf},
};

fn main() {
    println!("Hello, world!");
}
