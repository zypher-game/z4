use std::ops::{Add, Mul, Sub};

use crate::errors::{PokerError, Result};
use ark_ec::{AffineRepr, CurveGroup};
use ark_ed_on_bn254::EdwardsAffine;
use ark_ff::{BigInteger, PrimeField};
use ark_std::UniformRand;
use rand_chacha::rand_core::{CryptoRng, RngCore};
use zplonk::anemoi::{AnemoiJive, AnemoiJive254};

/// The signing key.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SigningKey(ark_ed_on_bn254::Fr);

/// The verifying key.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerifyingKey(ark_ed_on_bn254::EdwardsAffine);

/// The signature.
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct Signature {
    pub s: ark_ed_on_bn254::Fr,
    pub e: ark_bn254::Fr,
}

/// The keypair for schnorr signature.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KeyPair {
    pub(crate) signing_key: SigningKey,
    pub(crate) verifying_key: VerifyingKey,
}

impl KeyPair {
    pub fn sample<R: CryptoRng + RngCore>(prng: &mut R) -> Self {
        let sk = SigningKey::random(prng);
        Self::from_signing_key(sk)
    }

    pub fn from_signing_key(sk: SigningKey) -> Self {
        let vk = ark_ed_on_bn254::EdwardsAffine::generator().mul(&sk.0);
        Self {
            signing_key: sk,
            verifying_key: VerifyingKey(vk.into_affine()),
        }
    }

    /// Get the private key.
    pub fn get_private_key(&self) -> SigningKey {
        self.signing_key.clone()
    }

    /// Get the public key.
    pub fn get_public_key(&self) -> VerifyingKey {
        self.verifying_key.clone()
    }

    pub fn sign<R: CryptoRng + RngCore>(
        &self,
        msg: &[ark_bn254::Fr],
        prng: &mut R,
    ) -> Result<Signature> {
        let r = ark_ed_on_bn254::Fr::rand(prng);
        let big_r = EdwardsAffine::generator().mul(&r).into_affine();

        let mut input = vec![
            self.get_public_key().0.x,
            self.get_public_key().0.y,
            big_r.x,
            big_r.y,
        ];
        input.extend_from_slice(msg);

        let e = AnemoiJive254::eval_variable_length_hash(&input);

        let e_reduction =
            ark_ed_on_bn254::Fr::from_be_bytes_mod_order(&e.into_bigint().to_bytes_be());

        let s = r.sub(&self.get_private_key().0.mul(e_reduction));

        Ok(Signature { s, e })
    }
}

impl SigningKey {
    pub fn random<R: CryptoRng + RngCore>(prng: &mut R) -> Self {
        let sk = ark_ed_on_bn254::Fr::rand(prng);
        Self(sk)
    }
}

impl VerifyingKey {
    pub fn verify(&self, s: &Signature, msg: &[ark_bn254::Fr]) -> Result<()> {
        let e_reduction =
            ark_ed_on_bn254::Fr::from_be_bytes_mod_order(&s.e.into_bigint().to_bytes_be());
        let big_r = EdwardsAffine::generator()
            .mul(&s.s)
            .add(self.0.mul(&e_reduction))
            .into_affine();

        let mut input = vec![self.0.x, self.0.y, big_r.x, big_r.y];
        input.extend_from_slice(msg);

        let e = AnemoiJive254::eval_variable_length_hash(&input);

        if e != s.e {
            Err(PokerError::VerifySignatureError)
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod test {
    use crate::schnorr::KeyPair;
    use ark_std::UniformRand;
    use rand_chacha::{rand_core::SeedableRng, ChaChaRng};

    #[test]
    fn test_schnorr() {
        let mut prng = ChaChaRng::from_seed([0u8; 32]);
        let key_pair = KeyPair::sample(&mut prng);
        let msg = vec![
            ark_bn254::Fr::rand(&mut prng),
            ark_bn254::Fr::rand(&mut prng),
            ark_bn254::Fr::rand(&mut prng),
        ];
        let s = key_pair.sign(&msg, &mut prng).unwrap();
        assert!(key_pair.get_public_key().verify(&s, &msg).is_ok());
    }
}
