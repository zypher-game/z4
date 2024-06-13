use serde::{Deserialize, Serialize};
use uzkge::errors::UzkgeError;

use crate::PeerId;

/// Z4 error
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

/// Export ethers Address and H160
pub use ethers::prelude::{Address, H160};

/// Z4 main Result with Z4 error
pub type Result<T> = core::result::Result<T, Error>;

/// Room id = u64
pub type RoomId = u64;

/// Game id = Address
pub type GameId = Address;

/// Z4 init room id/tdn group id
pub const Z4_ROOM_MARKET_GROUP: RoomId = 4;

/// Convert address to hex string
pub fn address_hex(a: &Address) -> String {
    PeerId(a.to_fixed_bytes()).to_hex()
}

/// Convert hex string to address
pub fn hex_address(v: &str) -> Result<Address> {
    if let Ok(v) = hex::decode(v.trim_start_matches("0x")) {
        if v.len() != 20 {
            return Err(Error::Anyhow("address invalid".to_owned()));
        }
        let mut bytes = [0u8; 20];
        bytes.copy_from_slice(&v);
        Ok(H160(bytes))
    } else {
        Err(Error::Anyhow("address invalid".to_owned()))
    }
}

/// Get value from env and parse it to T
pub fn env_value<T: std::str::FromStr>(key: &str, default: Option<T>) -> Result<T> {
    match (std::env::var(key), default) {
        (Ok(v), _) => v
            .parse()
            .map_err(|_| Error::Anyhow(key.to_owned() + " env invalid")),
        (Err(_), Some(v)) => Ok(v),
        (Err(_), None) => return Err(Error::Anyhow(key.to_owned() + " env missing")),
    }
}

/// Get value array from env and parse it to T
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

/// P2P network message type
#[derive(Serialize, Deserialize)]
pub struct P2pMessage<'a> {
    pub method: &'a str,
    pub params: Vec<u8>,
}

/// The message type synced from chain
pub enum ChainMessage {
    /// create a room on the chain,
    /// room_id, game_id, viewable, player account, player peer id, player pubkey,
    /// salt by player, current block prevrandao
    CreateRoom(
        RoomId,
        GameId,
        bool,
        Address,
        PeerId,
        [u8; 32],
        [u8; 32],
        [u8; 32],
    ),
    /// join a room on the chain,
    /// room_id, player account, player peer id, player pubkey
    JoinRoom(RoomId, Address, PeerId, [u8; 32]),
    /// start a room on the chain,
    /// room_id, game address
    StartRoom(RoomId, Address),
    /// accept a room on the chain,
    /// room_id, sequencer account, sequencer websocket, params when accept
    AcceptRoom(RoomId, PeerId, String, Vec<u8>),
    /// game over in local,
    /// room_id, result, proof
    GameOverRoom(RoomId, Vec<u8>, Vec<u8>),
    /// game over on the chain,
    /// room_id
    ChainOverRoom(RoomId),
    /// need reprove in local
    Reprove,
}

/// The message type when send to pool
pub enum PoolMessage {
    /// send a transaction to accept room,
    /// room_id, params
    AcceptRoom(RoomId, Vec<u8>),
    /// send a transaction to over room,
    /// room_id, result, proof
    OverRoom(RoomId, Vec<u8>, Vec<u8>),
    /// the transaction had been submmited
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
