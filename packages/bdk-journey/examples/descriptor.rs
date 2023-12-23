use bdk::{bitcoin::Network, database::MemoryDatabase, wallet::AddressIndex, Wallet};

/// Generate new address from descriptors
fn main() {
    let external_desc = "wpkh(tprv8ZgxMBicQKsPdy6LMhUtFHAgpocR8GC6QmwMSFpZs7h6Eziw3SpThFfczTDh5rW2krkqffa11UpX3XkeTTB2FvzZKWXqPY54Y6Rq4AQ5R8L/84'/0'/0'/0/*)";
    let internal_desc = "wpkh(tprv8ZgxMBicQKsPdy6LMhUtFHAgpocR8GC6QmwMSFpZs7h6Eziw3SpThFfczTDh5rW2krkqffa11UpX3XkeTTB2FvzZKWXqPY54Y6Rq4AQ5R8L/84'/0'/0'/1/*)";

    let wallet = Wallet::new(
        external_desc,
        Some(internal_desc),
        Network::Testnet,
        MemoryDatabase::default(),
    )
    .expect("Can't create wallet!");

    let address = wallet
        .get_address(AddressIndex::New)
        .expect("Can't create new address");
    println!("Generated address: {:?}", address.address);
}
