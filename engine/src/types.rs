use tdn::prelude::PeerId;

pub enum Error {
    //
}

pub type Result<T> = core::result::Result<T, Error>;

pub type RoomId = u64;
pub type Account = PeerId;

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
