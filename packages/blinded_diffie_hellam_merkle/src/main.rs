use blinded_dhm::{MintCrypto, ProviderCrypto};

fn main() {
    let mint = MintCrypto::new();

    let provider = ProviderCrypto::new(mint.public_key());

    let blinded_token = mint.blinded_token(provider.blinded_message());

    let unblinded_token = provider.unblind(blinded_token);

    assert!(mint.proof(unblinded_token));
}
