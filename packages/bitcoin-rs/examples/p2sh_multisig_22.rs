use bitcoin_rs::legacy_multisig;
use hex_lit::hex;

fn main() {
    // Spending transaction:
    // bitcoin-cli getrawtransaction 214646c4b563cd8c788754ec94468ab71602f5ed07d5e976a2b0e41a413bcc0e 3
    // After decoding ScriptSig from the input: 0, its last ASM element it the scriptPubkey:
    // bitcoin-cli decodescript 5221032d7306898e980c66aefdfb6b377eaf71597c449bf9ce741a3380c5646354f6de2103e8c742e1f283ef810c1cd0c8875e5c2998a05fc5b23c30160d3d33add7af565752ae
    // its ASM is 2 of 2 multisig: 2 032d7306898e980c66aefdfb6b377eaf71597c449bf9ce741a3380c5646354f6de 03e8c742e1f283ef810c1cd0c8875e5c2998a05fc5b23c30160d3d33add7af5657 2 OP_CHECKMULTISIG

    let raw_tx = hex!("0100000001d611ad58b2f5bc0db7d15dfde4f497d6482d1b4a1e8c462ef077d4d32b3dae7901000000da0047304402203b17b4f64fa7299e8a85a688bda3cb1394b80262598bbdffd71dab1d7f266098022019cc20dc20eae417374609cb9ca22b28261511150ed69d39664b9d3b1bcb3d1201483045022100cfff9c400abb4ce5f247bd1c582cf54ec841719b0d39550b714c3c793fb4347b02201427a961a7f32aba4eeb1b71b080ea8712705e77323b747c03c8f5dbdda1025a01475221032d7306898e980c66aefdfb6b377eaf71597c449bf9ce741a3380c5646354f6de2103e8c742e1f283ef810c1cd0c8875e5c2998a05fc5b23c30160d3d33add7af565752aeffffffff020ed000000000000016001477800cff52bd58133b895622fd1220d9e2b47a79cd0902000000000017a914da55145ca5c56ba01f1b0b98d896425aa4b0f4468700000000");
    let input_idx = 0;

    println!("\nsighash_p2sh_multisig_2x2:");
    legacy_multisig::verify_signature_legacy(&raw_tx, input_idx, None);
}