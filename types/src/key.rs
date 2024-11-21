use ark_ec::PrimeGroup;
use ark_ed_on_bn254::{EdwardsAffine, EdwardsProjective, Fr};
use ark_std::{
    rand::{CryptoRng, RngCore},
    UniformRand,
};

/// Type PublicKey
pub type PublicKey = EdwardsAffine;

/// Type SecretKey
pub type SecretKey = Fr;

/// Generate engine used keypair for player
pub fn generate_keypair<R: CryptoRng + RngCore>(prng: &mut R) -> (SecretKey, PublicKey) {
    let sk = Fr::rand(prng);
    let pk = EdwardsProjective::generator() * sk;
    (sk, EdwardsAffine::from(pk))
}

/// Sign for zk-friendly
pub fn sign() {
    //
}

/// Verify for zk-friendly
pub fn verify() {
    //
}
