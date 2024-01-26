use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{cards::CryptoCard, play::PlayerEnv, schnorr::PublicKey};
use serde_with::serde_as;

#[serde_as]
#[derive(Debug, Deserialize, Serialize)]
pub struct Task {
    pub room_id: usize,
    pub num_round: usize,
    pub players_order: Vec<PublicKey>,
    pub players_env: Vec<Vec<PlayerEnv>>,
    #[serde_as(as = "Vec<(_, _)>")]
    pub players_hand: HashMap<PublicKey, Vec<CryptoCard>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TaskCommit {
    pub room_id: usize,
    pub players_order: Vec<PublicKey>,
    pub players_hand: HashMap<PublicKey, Vec<CryptoCard>>,
    pub winner: PublicKey,
}
