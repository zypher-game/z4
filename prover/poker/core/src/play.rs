use crate::{
    cards::EncodingCard,
    combination::CryptoCardCombination,
    errors::{PokerError, Result},
    schnorr::{KeyPair, PublicKey, Signature},
};
use ark_bn254::Fr;
use rand_chacha::rand_core::{CryptoRng, RngCore};
use serde::{Deserialize, Serialize};
use zshuffle::{
    reveal::{unmask, verify_reveal},
    RevealProof,
};

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
pub enum PlayAction {
    PAAS,
    PLAY,
}

impl From<PlayAction> for u8 {
    fn from(val: PlayAction) -> Self {
        match val {
            PlayAction::PAAS => 0,
            PlayAction::PLAY => 1,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PlayerEnv {
    // The unique identifier for the game room.
    pub room_id: usize,

    // The identifier for the current game round.
    pub round_id: usize,

    // The identifier for the current turn within the round.
    pub turn_id: usize,
    pub action: PlayAction,
    pub play_cards: Option<CryptoCardCombination>,
    pub owner_reveal: Vec<(EncodingCard, RevealProof, PublicKey)>,
    pub others_reveal: Vec<Vec<(EncodingCard, RevealProof, PublicKey)>>,
    // Currently using schnorr signatures, with plans to transition to aggregated signatures in the future.
    pub signature: Signature,
}

impl Default for PlayerEnv {
    fn default() -> Self {
        Self {
            room_id: 0,
            turn_id: 0,
            round_id: 0,
            action: PlayAction::PAAS,
            play_cards: None,
            owner_reveal: vec![],
            others_reveal: vec![],
            signature: Signature::default(),
        }
    }
}

/// A builder used to construct an [PlayerEnv].
#[derive(Default)]
pub struct PlayerEnvBuilder {
    pub(crate) inner: PlayerEnv,
}

impl PlayerEnv {
    /// Construct a [PlayerEnvBuilder].
    pub fn builder() -> PlayerEnvBuilder {
        PlayerEnvBuilder::default()
    }

    pub fn verify_sign(&self, pk: &PublicKey) -> Result<()> {
        let mut msg = vec![
            Fr::from(self.room_id as u64),
            Fr::from(self.round_id as u64),
            Fr::from(self.turn_id as u64),
            Fr::from(Into::<u8>::into(self.action)),
        ];

        let cards = {
            if self.action != PlayAction::PAAS {
                self.play_cards.clone().unwrap().flatten()
            } else {
                vec![]
            }
        };

        msg.extend(cards);

        pk.verify(&self.signature, &msg)
    }

    pub fn verify_sign_with_params(
        &self,
        pk: &PublicKey,
        room_id: usize,
        round_id: usize,
        turn_id: usize,
    ) -> Result<()> {
        let mut msg = vec![
            Fr::from(room_id as u64),
            Fr::from(round_id as u64),
            Fr::from(turn_id as u64),
            Fr::from(Into::<u8>::into(self.action)),
        ];

        let cards = {
            if self.action != PlayAction::PAAS {
                self.play_cards.clone().unwrap().flatten()
            } else {
                vec![]
            }
        };

        msg.extend(cards);

        pk.verify(&self.signature, &msg)
    }

    pub fn verify_and_get_reveals(&self) -> Result<Vec<EncodingCard>> {
        let cards = self.play_cards.clone().ok_or(PokerError::NoCardError)?;
        let vec = cards.to_vec();
        assert_eq!(vec.len(), self.others_reveal.len());
        assert_eq!(vec.len(), self.owner_reveal.len());

        let mut unmasked_cards = Vec::new();

        for (others, (owner, card)) in self
            .others_reveal
            .iter()
            .zip(self.owner_reveal.iter().zip(vec.iter()))
        {
            let mut reveals = Vec::new();
            for reveal in others.iter() {
                verify_reveal(&reveal.2.get_raw(), &card.0, &reveal.0 .0, &reveal.1)
                    .map_err(|_| PokerError::VerifyReVealError)?;
                reveals.push(reveal.0 .0);
            }

            verify_reveal(&owner.2.get_raw(), &card.0, &owner.0 .0, &owner.1)
                .map_err(|_| PokerError::VerifyReVealError)?;
            reveals.push(owner.0 .0);

            let unmasked_card =
                unmask(&card.0, &reveals).map_err(|_| PokerError::UnmaskCardError)?;
            unmasked_cards.push(EncodingCard(unmasked_card));
        }

        Ok(unmasked_cards)
    }
}

impl PlayerEnvBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn room_id(mut self, room_id: usize) -> Self {
        self.inner.room_id = room_id;
        self
    }

    pub fn turn_id(mut self, turn_id: usize) -> Self {
        self.inner.turn_id = turn_id;
        self
    }

    pub fn round_id(mut self, round_id: usize) -> Self {
        self.inner.round_id = round_id;
        self
    }

    pub fn action(mut self, action: PlayAction) -> Self {
        self.inner.action = action;
        self
    }

    pub fn play_cards(mut self, play_cards: Option<CryptoCardCombination>) -> Self {
        self.inner.play_cards = play_cards;
        self
    }

    pub fn others_reveal(
        mut self,
        others_reveal: &[Vec<(EncodingCard, RevealProof, PublicKey)>],
    ) -> Self {
        self.inner.others_reveal = others_reveal.to_vec();
        self
    }

    pub fn owner_reveal(mut self, owner_reveal: &[(EncodingCard, RevealProof, PublicKey)]) -> Self {
        self.inner.owner_reveal = owner_reveal.to_vec();
        self
    }

    pub fn sanity_check(&self) -> Result<()> {
        match self.inner.action {
            PlayAction::PAAS => {
                if !self.inner.others_reveal.is_empty()
                    || !self.inner.owner_reveal.is_empty()
                    || self.inner.play_cards.is_some()
                {
                    Err(PokerError::BuildPlayEnvParamsError)
                } else {
                    Ok(())
                }
            }
            PlayAction::PLAY => {
                if let Some(c) = &self.inner.play_cards {
                    // todo check  self.inner.others_reveal.len = participant
                    if self.inner.others_reveal.len() != c.len()
                        || self.inner.owner_reveal.len() != c.len()
                    {
                        Err(PokerError::BuildPlayEnvParamsError)
                    } else {
                        Ok(())
                    }
                } else {
                    Err(PokerError::BuildPlayEnvParamsError)
                }
            }
        }
    }

    pub fn build_and_sign<R: CryptoRng + RngCore>(
        mut self,
        key: &KeyPair,
        prng: &mut R,
    ) -> Result<PlayerEnv> {
        self.sanity_check()?;

        let mut msg = vec![
            Fr::from(self.inner.room_id as u64),
            Fr::from(self.inner.round_id as u64),
            Fr::from(self.inner.turn_id as u64),
            Fr::from(Into::<u8>::into(self.inner.action)),
        ];

        let cards = {
            if self.inner.action != PlayAction::PAAS {
                self.inner.play_cards.clone().unwrap().flatten()
            } else {
                vec![]
            }
        };

        msg.extend(cards);

        let s = key.sign(&msg, prng)?;

        self.inner.signature = s;

        Ok(self.inner)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rand_chacha::{rand_core::SeedableRng, ChaChaRng};

    #[test]
    fn test_player() {
        let mut prng = ChaChaRng::from_seed([0u8; 32]);
        let key_pair = KeyPair::sample(&mut prng);
        let player = PlayerEnvBuilder::new()
            .room_id(1)
            .round_id(1)
            .turn_id(1)
            .action(PlayAction::PAAS)
            .build_and_sign(&key_pair, &mut prng)
            .unwrap();

        assert!(player.verify_sign(&key_pair.get_public_key()).is_ok());
    }
}
