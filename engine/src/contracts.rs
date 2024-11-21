use ethers::prelude::*;

// RoomMarket contract with abi
abigen!(RoomMarket, "public/ABI/RoomMarket.json");

// ERC20 Token contract with abi
abigen!(Token, "public/ABI/Token.json");

// SimpleGame contract with abi
abigen!(SimpleGame, "public/ABI/SimpleGame.json");
