use serde::{Deserialize, Serialize};
use uzkge::errors::UzkgeError;

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
    Zk(UzkgeError),
}

pub use ethers::prelude::{Address, H160};

pub type Result<T> = core::result::Result<T, Error>;

pub type RoomId = u64;

pub type GameId = Address;

pub const Z4_ROOM_MARKET_GROUP: RoomId = 4;

pub fn address_hex(a: &Address) -> String {
    PeerId(a.to_fixed_bytes()).to_hex()
}

pub fn hex_address(v: &str) -> Result<Address> {
    if let Ok(v) = hex::decode(v.trim_start_matches("0x")) {
        let mut bytes = [0u8; 20];
        bytes.copy_from_slice(&v);
        Ok(H160(bytes))
    } else {
        Err(Error::Anyhow("address invalid".to_owned()))
    }
}

pub fn env_value<T: std::str::FromStr>(key: &str, default: Option<T>) -> Result<T> {
    match (std::env::var(key), default) {
        (Ok(v), _) => v
            .parse()
            .map_err(|_| Error::Anyhow(key.to_owned() + " env invalid")),
        (Err(_), Some(v)) => Ok(v),
        (Err(_), None) => return Err(Error::Anyhow(key.to_owned() + " env missing")),
    }
}

pub fn env_values<T: std::str::FromStr>(key: &str, default: Option<Vec<T>>) -> Result<Vec<T>> {
    match (std::env::var(key), default) {
        (Ok(v), default) => {
            let mut items = vec![];
            for item in v.split(",") {
                items.push(
                    item.parse()
                        .map_err(|_| Error::Anyhow(key.to_owned() + " env invalid"))?,
                );
            }
            if items.is_empty() {
                if let Some(d) = default {
                    Ok(d)
                } else {
                    Err(Error::Anyhow(key.to_owned() + " env invalid"))
                }
            } else {
                Ok(items)
            }
        }
        (Err(_), Some(v)) => Ok(v),
        (Err(_), None) => return Err(Error::Anyhow(key.to_owned() + " env missing")),
    }
}

#[derive(Serialize, Deserialize)]
pub struct P2pMessage<'a> {
    pub method: &'a str,
    pub params: Vec<u8>,
}

pub enum ChainMessage {
    CreateRoom(RoomId, GameId, bool, Address, PeerId, [u8; 32]),
    JoinRoom(RoomId, Address, PeerId, [u8; 32]),
    StartRoom(RoomId, Address),
    AcceptRoom(RoomId, PeerId, String, Vec<u8>),
    GameOverRoom(RoomId, Vec<u8>, Vec<u8>),
    ChainOverRoom(RoomId),
    Reprove,
}

pub enum PoolMessage {
    AcceptRoom(RoomId, Vec<u8>),
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

impl From<UzkgeError> for Error {
    fn from(err: UzkgeError) -> Error {
        Error::Zk(err)
    }
}
