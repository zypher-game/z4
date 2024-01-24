use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{cards::EncodingCard, play::PlayerEnv, schnorr::PublicKey};

#[derive(Debug, Deserialize, Serialize)]
pub struct Task {
    pub room_id: usize,
    pub game_id: usize,
    pub num_round: usize,
    pub players_order: Vec<PublicKey>,
    pub players_env: Vec<Vec<PlayerEnv>>,
    pub players_hand: HashMap<PublicKey, Vec<EncodingCard>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TaskCommit {
    pub room_id: usize,
    pub game_id: usize,
    pub players_order: Vec<PublicKey>,
    pub players_hand: HashMap<PublicKey, Vec<EncodingCard>>,
    pub winner: PublicKey,
}
