use crate::{combination::Combination, ecdsa::Sign};

pub enum PlayAction {
    PAAS,
    Lead,
    FOLLOW,
}

pub struct Game {
    pub players: Vec<PlayerEnv>,
}

pub struct PlayerEnv {
    pub room_id: usize,
    pub round_num: usize,
    pub action: PlayAction,
    pub play_cards: Option<Combination>,
    pub owner_unmask: (),
    pub others_reveal: (),
    // Currently using ECDSA signatures, with plans to transition to aggregated signatures in the future.
    pub signature: Sign,
}

impl Default for PlayerEnv {
    fn default() -> Self {
        Self {
            room_id: 0,
            round_num: 0,
            action: PlayAction::PAAS,
            play_cards: None,
            owner_unmask: (),
            others_reveal: (),
            signature: todo!(), // Sign::default();
        }
    }
}

#[derive(Default)]
/// A builder used to construct an [PlayerEnv].
pub struct PlayerEnvBuilder {
    inner: PlayerEnv,
}

impl PlayerEnv {
    /// Construct a [PlayerEnvBuilder].
    pub fn builder() -> PlayerEnvBuilder {
        PlayerEnvBuilder::default()
    }
}
