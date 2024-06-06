use bitcoin_rs::legacy_multisig;
use hex_lit::hex;

fn main() {
    // Spending tx:
    // bitcoin-cli getrawtransaction 949591ad468cef5c41656c0a502d9500671ee421fadb590fbc6373000039b693 3
    // Input 0 scriptSig has 2 sigs
    let raw_tx = hex!("010000000110a5fee9786a9d2d72c25525e52dd70cbd9035d5152fac83b62d3aa7e2301d58000000009300483045022100af204ef91b8dba5884df50f87219ccef22014c21dd05aa44470d4ed800b7f6e40220428fe058684db1bb2bfb6061bff67048592c574effc217f0d150daedcf36787601483045022100e8547aa2c2a2761a5a28806d3ae0d1bbf0aeff782f9081dfea67b86cacb321340220771a166929469c34959daf726a2ac0c253f9aff391e58a3c7cb46d8b7e0fdc4801ffffffff0180a21900000000001976a914971802edf585cdbc4e57017d6e5142515c1e502888ac00000000");

    // Original transaction:
    // bitcoin-cli getrawtransaction 581d30e2a73a2db683ac2f15d53590bd0cd72de52555c2722d9d6a78e9fea510  3
    // Out 0 scriptPubKey.type “multisig” has 3 uncompressed pubkeys
    let reftx_script_pubkey_bytes = hex!("524104d81fd577272bbe73308c93009eec5dc9fc319fc1ee2e7066e17220a5d47a18314578be2faea34b9f1f8ca078f8621acd4bc22897b03daa422b9bf56646b342a24104ec3afff0b2b66e8152e9018fe3be3fc92b30bf886b3487a525997d00fd9da2d012dce5d5275854adc3106572a5d1e12d4211b228429f5a7b2f7ba92eb0475bb14104b49b496684b02855bc32f5daefa2e2e406db4418f3b86bca5195600951c7d918cdbe5e6d3736ec2abf2dd7610995c3086976b2c0c7b4e459d10b34a316d5a5e753ae");
    let input_idx = 0;

    println!("\nsighash_p2ms_multisig_2x3:");
    legacy_multisig::verify_signature_legacy(&raw_tx, input_idx, Some(&reftx_script_pubkey_bytes));
}
