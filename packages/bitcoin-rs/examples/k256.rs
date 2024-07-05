use k256::{
    ecdsa::{SigningKey, VerifyingKey},
    elliptic_curve::sec1::{FromEncodedPoint, ToEncodedPoint},
    ProjectivePoint,
};
use rand::{thread_rng, Rng};
fn main() {
    let mut rng = thread_rng();
    let s1 = rng.gen::<[u8; 32]>();
    let s2 = rng.gen::<[u8; 32]>();

    let s1_g = generate_public_key(&s1);
    let s2_g = generate_public_key(&s2);

    let combined_key = add_public_key(&s1_g, &s2_g);

    println!("s1: {:x?}", s1);
    println!("s2: {:x?}", s2);
    println!("s1_g: {:?}", s1_g);
    println!("s2_g: {:?}", s2_g);
    println!("(s1 + s2)G: {:?}", combined_key);
}

fn generate_public_key(nonce: &[u8; 32]) -> ProjectivePoint {
    let signing_key = SigningKey::from_bytes(nonce).expect("Invalid nonce");
    let verifying_key = VerifyingKey::from(&signing_key);
    let point = verifying_key.to_encoded_point(true);

    ProjectivePoint::from_encoded_point(&point).unwrap()
}

fn add_public_key(s1: &ProjectivePoint, s2: &ProjectivePoint) -> ProjectivePoint {
    *s1 + *s2
}
