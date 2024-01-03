#[derive(Debug)]
pub enum Error {
    /// Invalid params
    Params,
    /// not has the room
    NoRoom,
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
