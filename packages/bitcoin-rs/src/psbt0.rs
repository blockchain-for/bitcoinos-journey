use std::{collections::BTreeMap, str::FromStr};

use bitcoin::{
    absolute,
    bip32::{ChildNumber, Fingerprint, IntoDerivationPath, Xpriv, Xpub},
    consensus,
    psbt::Input,
    secp256k1::{Secp256k1, Signing},
    transaction, Address, Amount, EcdsaSighashType, Network, OutPoint, Psbt, ScriptBuf, Sequence,
    Transaction, TxIn, TxOut, Txid, WPubkeyHash, Witness,
};
use bitcoin_hashes::Hash;

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

fn receiver_address() -> Address {
    str_to_address(
        "bc1q7cyrfmck2ffu2ud3rn5l5a8yv6f0chkp0zpemf",
        Network::Bitcoin,
    )
}

pub fn dummy_utxos() -> Vec<(OutPoint, TxOut)> {
    let script_pubkey_1 = str_to_address(
        "bc1qrwuu3ydv0jfza4a0ehtfd03m9l4vw3fy0hfm50",
        Network::Bitcoin,
    )
    .script_pubkey();

    let out_point_1 = OutPoint {
        txid: Txid::all_zeros(),
        vout: 0,
    };

    let utxo_1 = TxOut {
        value: DUMMY_UTXO_AMOUNT_INPUT_1,
        script_pubkey: script_pubkey_1,
    };

    let script_pubkey_2 = str_to_address(
        "bc1qy7swwpejlw7a2rp774pa8rymh8tw3xvd2x2xkd",
        Network::Bitcoin,
    )
    .script_pubkey();

    let out_point_2 = OutPoint {
        txid: Txid::all_zeros(),
        vout: 1,
    };

    let utxo_2 = TxOut {
        value: DUMMY_UTXO_AMOUNT_INPUT_2,
        script_pubkey: script_pubkey_2,
    };

    vec![(out_point_1, utxo_1), (out_point_2, utxo_2)]
}

pub fn str_to_address(address: &str, network: Network) -> Address {
    Address::from_str(address)
        .expect("must be a valid bitcoin address")
        .require_network(network)
        .expect("address must match the network")
}

pub fn run() {
    let secp = Secp256k1::new();

    // Get the individual xprivs we control.
    // In a real application these should come from a secret store
    let master_xpriv = XPRIV.parse::<Xpriv>().expect("must be a valid xpriv");
    let xpriv_input_1 = get_extenal_address_xpriv(&secp, master_xpriv, 0);
    let xpriv_input_2 = get_internal_address_xpriv(&secp, master_xpriv, 0);
    let xpriv_change = get_internal_address_xpriv(&secp, master_xpriv, 1);

    // Get PubKeys
    let pubkey_input_1 = Xpub::from_priv(&secp, &xpriv_input_1).to_pub();
    let pubkey_input_2 = Xpub::from_priv(&secp, &xpriv_input_2).to_pub();
    let pubkey_inputs = [pubkey_input_1, pubkey_input_2];

    let pubkey_change = Xpub::from_priv(&secp, &xpriv_change).to_pub();

    // Get the Witness Public Key Hashes (WPKHs)
    let wpkhs: Vec<WPubkeyHash> = pubkey_inputs.iter().map(|p| p.wpubkey_hash()).collect();

    // Get the unspent outputs that are locked to the key above that we control
    // In a real application these Inputs would come from the chain (another UTXOs)
    let utxos: Vec<TxOut> = dummy_utxos().into_iter().map(|(_, utxo)| utxo).collect();

    // Get receiver address
    let receiver_address = receiver_address();

    let inputs: Vec<TxIn> = dummy_utxos()
        .into_iter()
        .map(|(outpoint, _)| TxIn {
            previous_output: outpoint,
            script_sig: ScriptBuf::default(),
            sequence: Sequence::ENABLE_LOCKTIME_NO_RBF,
            witness: Witness::default(),
        })
        .collect();

    let spend = TxOut {
        value: SPEND_AMOUNT,
        script_pubkey: receiver_address.script_pubkey(),
    };

    let change = TxOut {
        value: CHANGE_AMOUNT,
        script_pubkey: ScriptBuf::new_p2wpkh(&pubkey_change.wpubkey_hash()),
    };

    // the tx we want to sign and broadcast
    let unsigned_tx = Transaction {
        version: transaction::Version::TWO,  // Post BIP 68,
        lock_time: absolute::LockTime::ZERO, // Ignore the locktime
        input: inputs,
        output: vec![spend, change],
    };

    // Start the PSBT workflow
    // Step 1: Creator role: Creates and add inputs and outputs to the PSBT
    let mut psbt = Psbt::from_unsigned_tx(unsigned_tx).expect("Could not create PSBT");

    // Step 2: Updater role: Adds additional information to the PSBT
    let ty = EcdsaSighashType::All.into();

    let derivation_paths = [
        "m/84'/0'/0'/0/0"
            .into_derivation_path()
            .expect("valid derivation path"),
        "m/84'/0'/0'/0/1"
            .into_derivation_path()
            .expect("valid derivation path"),
    ];

    let mut bip32_derivations = Vec::new();

    for (idx, pubkey) in pubkey_inputs.iter().enumerate() {
        let mut map = BTreeMap::new();
        let fingerprint = Fingerprint::from_str(MASTER_FINGERPRINT).expect("valid fingerprint");
        map.insert(pubkey.0, (fingerprint, derivation_paths[idx].clone()));
        bip32_derivations.push(map);
    }

    psbt.inputs = vec![
        Input {
            witness_utxo: Some(utxos[0].clone()),
            redeem_script: Some(ScriptBuf::new_p2wpkh(&wpkhs[0])),
            bip32_derivation: bip32_derivations[0].clone(),
            sighash_type: Some(ty),
            ..Default::default()
        },
        Input {
            witness_utxo: Some(utxos[1].clone()),
            redeem_script: Some(ScriptBuf::new_p2wpkh(&wpkhs[1])),
            bip32_derivation: bip32_derivations[1].clone(),
            sighash_type: Some(ty),
            ..Default::default()
        },
    ];

    // Step 3: Signer role: Signs the PSBT
    psbt.sign(&master_xpriv, &secp)
        .expect("must be valid signature");

    // Step 4: Finalizer role: Finalizes the PSBT
    println!("PSBT Inputs: {:#?}", psbt.inputs);

    let final_script_witness: Vec<_> = psbt
        .inputs
        .iter()
        .enumerate()
        .map(|(idx, input)| {
            let (_, sig) = input.partial_sigs.iter().next().expect("Only has one sig");
            Witness::p2wpkh(sig, &pubkey_inputs[idx].0)
        })
        .collect();

    psbt.inputs.iter_mut().enumerate().for_each(|(idx, input)| {
        // Clear all the data fields as per the spec
        input.final_script_witness = Some(final_script_witness[idx].clone());
        input.partial_sigs = BTreeMap::new();
        input.sighash_type = None;
        input.redeem_script = None;
        input.witness_script = None;
        input.bip32_derivation = BTreeMap::new();
    });

    println!("PSBT: {:#?}", psbt);

    let signed_tx = psbt.extract_tx().expect("valid transaction");
    let serialized_signed_tx = consensus::encode::serialize_hex(&signed_tx);

    println!("Transaction Details: {:#?}", signed_tx);

    // bitcoin-cli decoderawtransaction <RAW_TX> true
    println!("Raw transaction: {}", serialized_signed_tx);
}
