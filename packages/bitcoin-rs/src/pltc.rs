use k256::{
    ecdsa::{SigningKey, VerifyingKey},
    elliptic_curve::{
        bigint::{ArrayEncoding, UInt},
        ops::Reduce,
        sec1::{FromEncodedPoint, ToEncodedPoint},
    },
    ProjectivePoint, Scalar, U256,
};
use rand::thread_rng;

pub struct PLTC {
    pub unlock_condition: Scalar,
}

#[derive(Debug, Default)]
pub struct Participant {}

impl Participant {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn create_pltc(&self, unlock_condition: Scalar) -> PLTC {
        PLTC { unlock_condition }
    }

    pub fn unlock_funds(&self, pltc: &PLTC, secret_value: Scalar) -> bool {
        pltc.unlock_condition == secret_value
    }
}
/// 生成公钥
pub fn pt_from_sk(signing_key: &SigningKey) -> ProjectivePoint {
    let verifying_key = VerifyingKey::from(signing_key);
    let point = verifying_key.to_encoded_point(true);

    ProjectivePoint::from_encoded_point(&point).unwrap()
}

pub fn pk_from_sk(sk: &Scalar) -> ProjectivePoint {
    let bytes = sk.to_bytes();
    let signing_key = SigningKey::from_bytes(bytes.as_slice()).unwrap();

    pt_from_sk(&signing_key)
}

pub fn sk_to_scalar(signing_key: &SigningKey) -> Scalar {
    let bytes = signing_key.to_bytes();
    <Scalar as Reduce<U256>>::from_uint_reduced(UInt::from_be_byte_array(bytes))
}

pub fn create_sk_scalar() -> Scalar {
    let rng = thread_rng();
    let sk = SigningKey::random(rng);
    sk_to_scalar(&sk)
}

pub fn create_ec_keypair() -> (Scalar, ProjectivePoint) {
    let rng = thread_rng();
    let sk = SigningKey::random(rng);
    let secret_key = sk_to_scalar(&sk);
    let public_key = pt_from_sk(&sk);

    (secret_key, public_key)
}
