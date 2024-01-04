use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum Error {
    /// Invalid params
    Params,
    /// not has the room
    NoRoom,
    /// serialize error
    Serialize,
    /// Anyhow error
    Anyhow(String),
}

pub type Result<T> = core::result::Result<T, Error>;

pub type RoomId = u64;

#[derive(Serialize, Deserialize)]
pub struct P2pMessage<'a> {
    pub method: &'a str,
    pub params: Vec<u8>,
}

pub enum ChainMessage {
    CreateRoom,
    JoinRoom,
    StartRoom,
    Reprove,
}

pub enum PoolMessage {
    Submitted,
    Submit,
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
