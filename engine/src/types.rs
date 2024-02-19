use serde::{Deserialize, Serialize};
use zplonk::errors::ZplonkError;

use crate::{PeerId, PublicKey};

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
    /// serialize error
    Serialize,
    /// invalid secret key
    SecretKey,
    /// Anyhow error
    Anyhow(String),
    /// ZK error,
    Zk(ZplonkError),
}

pub type Result<T> = core::result::Result<T, Error>;

pub type RoomId = u64;

#[derive(Serialize, Deserialize)]
pub struct P2pMessage<'a> {
    pub method: &'a str,
    pub params: Vec<u8>,
}

pub enum ChainMessage {
    StartRoom(RoomId, Vec<PeerId>, Vec<PublicKey>),
    AcceptRoom(RoomId, PeerId),
    OverRoom(RoomId, Vec<u8>, Vec<u8>),
    Reprove,
}

pub enum PoolMessage {
    AcceptRoom(RoomId),
    OverRoom(RoomId, Vec<u8>, Vec<u8>),
    Submitted(RoomId),
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
