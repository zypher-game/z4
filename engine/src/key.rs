use ark_ec::PrimeGroup;
use ark_ed_on_bn254::{EdwardsAffine, EdwardsProjective, Fr};
use ark_std::{
    rand::{CryptoRng, RngCore},
    UniformRand,
};

pub type PublicKey = EdwardsAffine;

pub type SecretKey = Fr;

/// generate engine used keypair for player
pub fn generate_keypair<R: CryptoRng + RngCore>(prng: &mut R) -> (SecretKey, PublicKey) {
    let sk = Fr::rand(prng);
    let pk = EdwardsProjective::generator() * sk;
    (sk, EdwardsAffine::from(pk))
}

/// sign for zk-friendly
pub fn sign() {
    //
}

/// verify for zk-friendly
pub fn verify() {
    //
}
