use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum Error {
    #[error("Num {num} not in field range 0 to {prime}")]
    ValueError { num: i64, prime: i64 },
    #[error("Can't operator with two different FieldElment {prime1} and {prime2}")]
    PrimeNotSameError { prime1: i64, prime2: i64 },
}
