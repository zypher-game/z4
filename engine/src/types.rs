#[derive(Debug)]
pub enum Error {
    /// Invalid params
    Params,
}

pub type Result<T> = core::result::Result<T, Error>;

pub type RoomId = u64;

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
