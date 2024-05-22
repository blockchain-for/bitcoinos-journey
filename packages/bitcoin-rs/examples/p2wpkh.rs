
use bitcoin_rs::p2wpkh;
use hex_lit::hex;

fn main() {
    test_sighash_p2wpkh()
}

fn test_sighash_p2wpkh() {
    // Spending transaction:
    // bitcoin-cli getrawtransaction 663becacc6368150a46725e404ccdfa34d1fffbececa784c31f0a7849b4dad08
    let raw_tx = hex!("020000000001015ce1d4ffc716022f83cc0d557e6dad0500eeff9e9623bde014bdc09c5b672d750000000000fdffffff025fb7460b000000001600142cf4c1dc0352e0658971ca62a7457a1cd8c3389c4ce3a2000000000016001433f57fe374c6ceab61c8639128c038ac2a8c8db60247304402203cb50efb5c4a9aa7fd369ab6f4b226db99f44f9c610b5b50bc42f343a6aa401302201af791542eee6c1b11705e8895cc5adc36458910dc91aadcafb76a6478a29b9f01210242e811e66fd17e9a6e4ef772766c668d6e0595ca1d7f0583148bc460b575fbfdf0df0b00");

    println!("raw tx is: {raw_tx:?}");

    // vin:0
    let input_idx = 0;

    // output value from the referenced vout:0 from the referenced tx:
    // bitcoin-cli getrawtransaction 752d675b9cc0bd14e0bd23969effee0005ad6d7e550dcc832f0216c7ffd4e15c  3
    let ref_out_value = 200_000_000;

    println!("\nsignhash_p2wpkh:");
    
    p2wpkh::verify_signature(&raw_tx, input_idx, ref_out_value);
}