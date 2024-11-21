use ethers::{
    abi::{encode, Token as AbiToken},
    prelude::*,
};

/// Export z4-type network and config
pub use z4_types::contracts::{Network, NetworkConfig};

// RoomMarket contract with abi
abigen!(RoomMarket, "public/ABI/RoomMarket.json");

// ERC20 Token contract with abi
abigen!(Token, "public/ABI/Token.json");

// SimpleGame contract with abi
abigen!(SimpleGame, "public/ABI/SimpleGame.json");

/// Helper for generate simple game result, for ranking
pub fn simple_game_result(ranks: &[Address]) -> Vec<u8> {
    encode(&[AbiToken::Array(
        ranks.iter().map(|v| AbiToken::Address(*v)).collect(),
    )])
}
