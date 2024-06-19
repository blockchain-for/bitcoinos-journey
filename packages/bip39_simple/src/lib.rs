pub mod domain;

/// Number of iterations to run by the PBKDF2 for key derivation
pub const ITERATION_COUNT: u32 = 2048;

/// The word used as a prefix for the salt for our key derivation function
pub const SALT_PREFIX: &str = "mnemonic";
