use ethers::prelude::*;

pub use z4_types::contracts::{Network, NetworkConfig};

abigen!(RoomMarket, "../public/ABI/RoomMarket.json");
abigen!(Token, "../public/ABI/Token.json");
abigen!(SimpleGame, "../public/ABI/SimpleGame.json");
