use bitcoin_rs::p2wsh_multisig_22;
use hex_lit::hex;

fn main() {
    // The spending transaction is
    // bitcoin-cli getrawtransaction 2bb157363e7a62d70b92082a9b2c9bb6f329154f816b8d239bd58c35c789a96a  3
    // input 0
    // ScriptPubkey from its witness data is:
    // bitcoin-cli decodescript 52210289da5da9d3700156db2d01e6362491733f6c886971791deda74b4e9d707190b2210323c437f30384498be79df2990ce5a8de00844e768c0ccce914335b6c26adea7352ae
    // its ASM is 2 0289da5da9d3700156db2d01e6362491733f6c886971791deda74b4e9d707190b2 0323c437f30384498be79df2990ce5a8de00844e768c0ccce914335b6c26adea73 2 OP_CHECKMULTISIG

    let raw_tx = hex!("010000000001011b9eb4122976fad8f809ee4cea8ac8d1c5b6b8e0d0f9f93327a5d78c9a3945280000000000ffffffff02ba3e0d00000000002200201c3b09401aaa7c9709d118a75d301bdb2180fb68b2e9b3ade8ad4ff7281780cfa586010000000000220020a41d0d894799879ca1bd88c1c3f1c2fd4b1592821cc3c5bfd5be5238b904b09f040047304402201c7563e876d67b5702aea5726cd202bf92d0b1dc52c4acd03435d6073e630bac022032b64b70d7fba0cb8be30b882ea06c5f8ec7288d113459dd5d3e294214e2c96201483045022100f532f7e3b8fd01a0edc86de4870db4e04858964d0a609df81deb99d9581e6c2e02206d9e9b6ab661176be8194faded62f518cdc6ee74dba919e0f35d77cff81f38e5014752210289da5da9d3700156db2d01e6362491733f6c886971791deda74b4e9d707190b2210323c437f30384498be79df2990ce5a8de00844e768c0ccce914335b6c26adea7352ae00000000");

    // For the witness transaction sighash computation, we need its referenced output's value from the original transaction.
    // bitcoin-cli getrawtransaction 2845399a8cd7a52733f9f9d0e0b8b6c5d1c88aea4cee09f8d8fa762912b49e1b  3
    // we need vout 0 value in sats:
    let ref_out_value = 968_240;

    println!("\nsighash_p2wsh_multisig_2x2:");

    p2wsh_multisig_22::verify_signature(&raw_tx, 0, ref_out_value);
}
