use ethers::{
    abi::{encode, Token as AbiToken},
    prelude::*,
};

pub use z4_types::contracts::{Network, NetworkConfig};

abigen!(RoomMarket, "public/ABI/RoomMarket.json");
abigen!(Token, "public/ABI/Token.json");
abigen!(SimpleGame, "public/ABI/SimpleGame.json");

/// helper for generate simple game result, for ranking
pub fn simple_game_result(ranks: &[Address]) -> Vec<u8> {
    encode(&[AbiToken::Array(
        ranks.iter().map(|v| AbiToken::Address(*v)).collect(),
    )])
}
