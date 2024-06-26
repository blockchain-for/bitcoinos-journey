use bitcoin::{
    bip32::{ChildNumber, IntoDerivationPath, Xpriv},
    secp256k1::{Secp256k1, Signing},
    Amount,
};

/// XPRIV is the extended private key that will be used to derive the keys for the SegWit V0 inputs
pub const XPRIV: &str = "xprv9tuogRdb5YTgcL3P8Waj7REqDuQx4sXcodQaWTtEVFEp6yRKh1CjrWfXChnhgHeLDuXxo2auDZegMiVMGGxwxcrb2PmiGyCngLxvLeGsZRq";
pub const BIP84_DERIVATION_PATH: &str = "m/84'/0'/0'";

/// MASTER_FINGERPRINT is the fingerprint of the master key
pub const MASTER_FINGERPRINT: &str = "9680603f";

const DUMMY_UTXO_AMOUNT_INPUT_1: Amount = Amount::from_sat(20_000_000);
const DUMMY_UTXO_AMOUNT_INPUT_2: Amount = Amount::from_sat(10_000_000);
const SPEND_AMOUNT: Amount = Amount::from_sat(25_000_000);
const CHANGE_AMOUNT: Amount = Amount::from_sat(4_990_000); // 10_000 sat fee.

pub fn get_extenal_address_xpriv<C: Signing>(
    secp: &Secp256k1<C>,
    master_xpriv: Xpriv,
    index: u32,
) -> Xpriv {
    // let derivation_path = BIP84_DERIVATION_PATH.into_derivation_path().expect("valid derivation path");

    // let child_xpriv = master_xpriv.derive_priv(secp, &derivation_path).expect("valid child xpriv");

    // let external_index = ChildNumber::from_normal_idx(0).unwrap();

    // let idx = ChildNumber::from_normal_idx(index).expect("valid index number");

    // child_xpriv.derive_priv(secp, &[external_index, idx])
    //     .expect("valid priv")
    get_address_xpriv(secp, master_xpriv, 0, index)
}

pub fn get_internal_address_xpriv<C: Signing>(
    secp: &Secp256k1<C>,
    master_xpriv: Xpriv,
    index: u32,
) -> Xpriv {
    get_address_xpriv(secp, master_xpriv, 1, index)
}

fn get_address_xpriv<C: Signing>(
    secp: &Secp256k1<C>,
    master_xpriv: Xpriv,
    child_number: u32,
    index: u32,
) -> Xpriv {
    let derivation_path = BIP84_DERIVATION_PATH
        .into_derivation_path()
        .expect("valid derivation path");

    let child_xpriv = master_xpriv
        .derive_priv(secp, &derivation_path)
        .expect("valid child xpriv");

    let child_number = ChildNumber::from_normal_idx(child_number).unwrap();

    let idx = ChildNumber::from_normal_idx(index).expect("valid index number");

    child_xpriv
        .derive_priv(secp, &[child_number, idx])
        .expect("valid priv")
}
