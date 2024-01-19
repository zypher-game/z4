use std::collections::HashMap;

use crate::{
    combination::CryptoCardCombination,
    errors::{PokerError, Result},
    schnorr::{KeyPair, Signature},
};
use ark_bn254::Fr;
use rand_chacha::rand_core::{CryptoRng, RngCore};
use zshuffle::{keygen::PublicKey, RevealCard, RevealProof};

#[derive(Debug, Clone, Copy, PartialEq)]
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

pub struct Task {
    pub room_id: usize,
    pub game_id: usize,
    pub num_round: usize,
    pub players_order: Vec<PublicKey>,
    pub games_env: Vec<Vec<PlayerEnv>>,
    pub players_hand: HashMap<PublicKey, Vec<zshuffle::Card>>,
}

pub struct PlayerEnv {
    // The unique identifier for the game room.
    pub room_id: usize,
    // The identifier for the current game round.
    pub game_id: usize,
    // The identifier for the current turn within the round.
    pub round_id: usize,
    pub action: PlayAction,
    pub play_cards: Option<CryptoCardCombination>,
    pub owner_reveal: Vec<(RevealCard, RevealProof, PublicKey)>,
    pub others_reveal: Vec<Vec<(RevealCard, RevealProof, PublicKey)>>,
    // Currently using schnorr signatures, with plans to transition to aggregated signatures in the future.
    pub signature: Signature,
}

impl Default for PlayerEnv {
    fn default() -> Self {
        Self {
            room_id: 0,
            game_id: 0,
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
}

impl PlayerEnvBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn room_id(mut self, room_id: usize) -> Self {
        self.inner.room_id = room_id;
        self
    }

    pub fn game_id(mut self, round_num: usize) -> Self {
        self.inner.game_id = round_num;
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
                if !self.inner.others_reveal.is_empty()
                    || !self.inner.owner_reveal.is_empty()
                    || self.inner.play_cards.is_some()
                {
                    Err(PokerError::BuildPlayEnvParasError)
                } else {
                    Ok(())
                }
            }
            PlayAction::PLAY => {
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
        prng: &mut R,
    ) -> Result<PlayerEnv> {
        self.sanity_check()?;

        let mut msg = vec![
            Fr::from(self.inner.room_id as u64),
            Fr::from(self.inner.game_id as u64),
            Fr::from(self.inner.round_id as u64),
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
    use rand_chacha::{rand_core::SeedableRng, ChaChaRng};

    use super::*;

    #[test]
    fn test_player() {
        let mut prng = ChaChaRng::from_seed([0u8; 32]);
        let key_pair = KeyPair::sample(&mut prng);
        let player = PlayerEnvBuilder::new()
            .room_id(1)
            .game_id(1)
            .round_id(1)
            .action(PlayAction::PAAS)
            .build_and_sign(&key_pair, &mut prng)
            .unwrap();

        let msg = vec![
            Fr::from(player.room_id as u64),
            Fr::from(player.game_id as u64),
            Fr::from(player.round_id as u64),
            Fr::from(Into::<u8>::into(player.action)),
        ];
        assert!(key_pair
            .get_public_key()
            .verify(&player.signature, &msg)
            .is_ok());
    }
}
