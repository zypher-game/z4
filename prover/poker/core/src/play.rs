use crate::{
    combination::CryptoCardCombination,
    errors::{PokerError, Result},
    schnorr::{KeyPair, Signature},
};
use ark_bn254::Fr;
use ark_ec::{AffineRepr, CurveGroup};
use rand_chacha::rand_core::{CryptoRng, RngCore};
use zshuffle::{keygen::PublicKey, RevealCard, RevealProof};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlayAction {
    PAAS,
    Lead,
    FOLLOW,
}

impl From<PlayAction> for u8 {
    fn from(val: PlayAction) -> Self {
        match val {
            PlayAction::PAAS => 0,
            PlayAction::Lead => 1,
            PlayAction::FOLLOW => 2,
        }
    }
}

pub struct Task {
    pub players: Vec<PlayerEnv>,
}

pub struct PlayerEnv {
    // The unique identifier for the game room.
    pub room_id: usize,
    // The identifier for the current game round.
    pub round_id: usize,
    // The identifier for the current turn within the round.
    pub turn_id: usize,
    pub action: PlayAction,
    pub play_cards: Option<CryptoCardCombination>,
    pub owner_reveal: Vec<(RevealCard, RevealProof, PublicKey)>,
    pub others_reveal: Vec<Vec<(RevealCard, RevealProof, PublicKey)>>,
    // Currently using ECDSA signatures, with plans to transition to aggregated signatures in the future.
    pub signature: Signature,
}

impl Default for PlayerEnv {
    fn default() -> Self {
        Self {
            room_id: 0,
            round_id: 0,
            turn_id: 0,
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
}

impl PlayerEnvBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn room_id(mut self, room_id: usize) -> Self {
        self.inner.room_id = room_id;
        self
    }

    pub fn round_id(mut self, round_num: usize) -> Self {
        self.inner.round_id = round_num;
        self
    }

    pub fn turn_id(mut self, turn_id: usize) -> Self {
        self.inner.turn_id = turn_id;
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
        others_reveal: &[Vec<(RevealCard, RevealProof, PublicKey)>],
    ) -> Self {
        self.inner.others_reveal = others_reveal.to_vec();
        self
    }

    pub fn owner_reveal(mut self, owner_reveal: &[(RevealCard, RevealProof, PublicKey)]) -> Self {
        self.inner.owner_reveal = owner_reveal.to_vec();
        self
    }

    pub fn sanity_check(&self) -> Result<()> {
        match self.inner.action {
            PlayAction::PAAS => {
                if !self.inner.others_reveal.is_empty() || !self.inner.owner_reveal.is_empty() {
                    Err(PokerError::BuildPlayEnvParasError)
                } else {
                    Ok(())
                }
            }
            PlayAction::Lead | PlayAction::FOLLOW => {
                if let Some(c) = &self.inner.play_cards {
                    // todo check  self.inner.others_reveal.len = participant
                    if self.inner.others_reveal.iter().all(|x| x.len() == c.len())
                        || self.inner.owner_reveal.len() != c.len()
                    {
                        Err(PokerError::BuildPlayEnvParasError)
                    } else {
                        Ok(())
                    }
                } else {
                    Err(PokerError::BuildPlayEnvParasError)
                }
            }
        }
    }

    pub fn build_and_sign<R: CryptoRng + RngCore>(
        mut self,
        key: &KeyPair,
        reveal_cards: Option<Vec<zshuffle::Card>>,
        prng: &mut R,
    ) -> Result<PlayerEnv> {
        self.sanity_check()?;

        let mut cards = vec![];
        if self.inner.action != PlayAction::PAAS {
            if reveal_cards.is_none()
                || reveal_cards.clone().unwrap().len() != self.inner.owner_reveal.len()
            {
                return Err(PokerError::BuildPlayEnvParasError);
            }

            reveal_cards.unwrap().iter().for_each(|c| {
                let (x, y) = c.into_affine().xy().unwrap();
                cards.push(x);
                cards.push(y);
            });
        }

        let mut msg = vec![
            Fr::from(self.inner.room_id as u64),
            Fr::from(self.inner.round_id as u64),
            Fr::from(self.inner.turn_id as u64),
            Fr::from(Into::<u8>::into(self.inner.action)),
        ];
        msg.extend(cards);

        let s = key.sign(&msg, prng)?;

        self.inner.signature = s;

        Ok(self.inner)
    }
}

#[cfg(test)]
mod test {
    use rand_chacha::{rand_core::SeedableRng, ChaChaRng};

    use super::*;

    #[test]
    fn test() {
        let mut prng = ChaChaRng::from_seed([0u8; 32]);
        let key_pair = KeyPair::sample(&mut prng);
        let player = PlayerEnvBuilder::new()
            .room_id(1)
            .round_id(1)
            .turn_id(1)
            .action(PlayAction::PAAS)
            .build_and_sign(&key_pair, None, &mut prng)
            .unwrap();

        let msg = vec![
            Fr::from(player.room_id as u64),
            Fr::from(player.round_id as u64),
            Fr::from(player.turn_id as u64),
            Fr::from(Into::<u8>::into(player.action)),
        ];
        assert!(key_pair
            .get_public_key()
            .verify(&player.signature, &msg)
            .is_ok());
    }
}
