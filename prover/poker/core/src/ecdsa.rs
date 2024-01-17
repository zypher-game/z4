use k256::{
    ecdsa::{
        signature::{Signer, Verifier},
        Signature, SigningKey, VerifyingKey,
    },
    elliptic_curve::rand_core::{CryptoRng, RngCore},
};

/// The private key.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrivateKey(pub SigningKey);

/// The public key.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PublicKey(pub VerifyingKey);

/// The signature.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Sign(pub Signature);

/// The keypair for ecdsa signature.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KeyPair {
    /// The private key.
    pub(crate) private_key: PrivateKey,
    /// The public key.
    pub(crate) public_key: PublicKey,
}

impl KeyPair {
    pub fn sample<R: CryptoRng + RngCore>(prng: &mut R) -> Self {
        let sk = SigningKey::random(prng);
        let vk = VerifyingKey::from(&sk);

        KeyPair {
            private_key: PrivateKey(sk),
            public_key: PublicKey(vk),
        }
    }

    /// Get the private key.
    pub fn get_private_key(&self) -> PrivateKey {
        self.private_key.clone()
    }

    /// Get the public key.
    pub fn get_public_key(&self) -> PublicKey {
        self.public_key.clone()
    }
}

impl PrivateKey {
    pub fn sign(&self, m: &[u8]) -> Sign {
        Sign(self.0.sign(m))
    }
}

impl PublicKey {
    pub fn verify(&self, sign: &Sign, m: &[u8]) -> Result<(), k256::ecdsa::Error> {
        self.0.verify(m, &sign.0)
    }
}

#[cfg(test)]
mod test {
    use crate::ecdsa::KeyPair;
    use rand_chacha::{rand_core::SeedableRng, ChaChaRng};

    #[test]
    fn test_ecdsa() {
        let mut prng = ChaChaRng::from_seed([0u8; 32]);
        let key_pair = KeyPair::sample(&mut prng);
        let m = b"If I play the bomb (pair of Jokers), how would you respond?";
        let sign = key_pair.get_private_key().sign(m);
        assert!(key_pair.get_public_key().verify(&sign, m).is_ok());
    }
}
