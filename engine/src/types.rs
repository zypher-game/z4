use serde::{Deserialize, Serialize};
use zplonk::errors::ZplonkError;

use crate::PeerId;

#[derive(Debug)]
pub enum Error {
    /// Invalid params
    Params,
    /// Timeout
    Timeout,
    /// Not has the player
    NoPlayer,
    /// Not has the room
    NoRoom,
    /// Not support this game
    NoGame,
    /// serialize error
    Serialize,
    /// invalid secret key
    SecretKey,
    /// Anyhow error
    Anyhow(String),
    /// ZK error,
    Zk(ZplonkError),
}

pub use ethers::prelude::Address;

pub type Result<T> = core::result::Result<T, Error>;

pub type RoomId = u64;

pub type GameId = Address;

pub const INIT_ROOM_MARKET_GROUP: RoomId = 100000;

pub fn address_hex(a: &Address) -> String {
    PeerId(a.to_fixed_bytes()).to_hex()
}

#[derive(Serialize, Deserialize)]
pub struct P2pMessage<'a> {
    pub method: &'a str,
    pub params: Vec<u8>,
}

pub enum ChainMessage {
    CreateRoom(RoomId, GameId, Address, PeerId),
    JoinRoom(RoomId, Address, PeerId),
    StartRoom(RoomId, Address),
    AcceptRoom(RoomId, PeerId),
    OverRoom(RoomId, Vec<u8>, Vec<u8>),
    Reprove,
}

pub enum PoolMessage {
    AcceptRoom(RoomId),
    OverRoom(RoomId, Vec<u8>, Vec<u8>),
    Submitted(RoomId),
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::Anyhow(err.to_string())
    }
}

impl From<Box<bincode::ErrorKind>> for Error {
    fn from(_err: Box<bincode::ErrorKind>) -> Error {
        Error::Serialize
    }
}

impl From<serde_json::Error> for Error {
    fn from(_err: serde_json::Error) -> Error {
        Error::Serialize
    }
}

impl From<anyhow::Error> for Error {
    fn from(err: anyhow::Error) -> Error {
        Error::Anyhow(err.to_string())
    }
}

impl From<hex::FromHexError> for Error {
    fn from(_err: hex::FromHexError) -> Error {
        Error::Serialize
    }
}

impl From<ethers::prelude::WalletError> for Error {
    fn from(_err: ethers::prelude::WalletError) -> Error {
        Error::SecretKey
    }
}

impl From<ZplonkError> for Error {
    fn from(err: ZplonkError) -> Error {
        Error::Zk(err)
    }
}
