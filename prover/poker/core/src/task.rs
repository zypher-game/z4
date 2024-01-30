use crate::play::PlayerEnv;
use crate::{cards::CryptoCard, schnorr::PublicKey};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::collections::HashMap;
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
