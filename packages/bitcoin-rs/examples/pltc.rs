use bitcoin_rs::pltc::{create_ec_keypair, create_sk_scalar, pk_from_sk, Participant};

fn main() {
    let alice = Participant::new();
    let bob = Participant::new();
    let carol = Participant::new();

    // Carol set secret value, and share sG to Alice
    let (s, s_g) = create_ec_keypair();

    // Alice create pay channel to Bob, condition is: a + s
    let a = create_sk_scalar();
    let b = create_sk_scalar();
    // let c = create_sk_scalar();

    let alice_pltc_to_bob = alice.create_pltc(a + s);

    // Bob create pay channel to Carol, condition is: b + s
    let bob_pltc_to_carol = bob.create_pltc(a + b + s);

    // Carol  unlock funds
    let secret_value = a + b + s;
    assert!(carol.unlock_funds(&bob_pltc_to_carol, secret_value));
    println!("Carol successfully unlocked the funds.");

    // Bob unlock funds
    let bob_secret_value = secret_value - b;
    assert!(bob.unlock_funds(&alice_pltc_to_bob, bob_secret_value));
    println!("Bob successfully unlocked the funds.");

    // Alice prove paid to Carol
    let alice_secret_value = bob_secret_value - a;
    assert_eq!(pk_from_sk(&alice_secret_value), s_g);
    println!("Alice successfully proved paid to Carol.");
}
