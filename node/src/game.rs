use serde::{Deserialize, Serialize};
use z4_engine::{Result, SubGame};

#[derive(Serialize, Deserialize)]
pub struct GameLogic {
    pub constructor: Vec<u8>,
    pub methods: Vec<u8>,
    pub tasks: Vec<Vec<u8>>,
}

/// listening the chain & contract, when new game created, save it,
/// when game being deprecated, delete it or do nothing.
pub fn listen() {
    //
    todo!()
}

/// save to local storage file, name is game
pub async fn save(game: &SubGame, elf: &[u8]) {
    //
    todo!()
}

/// load from local storage file, name is game
pub async fn load(game: &SubGame) -> Result<GameLogic> {
    //
    todo!()
}

/// check game is exists
pub async fn contains(game: &SubGame) -> bool {
    //
    todo!()
}
