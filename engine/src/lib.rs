mod config;
mod engine;
mod p2p;
mod room;
mod rpc;
mod types;

#[cfg(feature = "request")]
pub mod request;

pub use config::Config;
pub use engine::Engine;
pub use serde_json::{json, Value};
pub use tdn::{
    prelude::{GroupId, Peer, PeerId, PeerKey, SendType},
    types::rpc::rpc_response,
};
pub use types::*;

/// The result when after handling the message or task.
#[derive(Default)]
pub struct HandleResult<P: Param> {
    /// need broadcast the msg in the room
    all: Vec<(String, P)>,
    /// need send to someone msg
    one: Vec<(PeerId, String, P)>,
    /// when game over, need prove the operations & states
    over: bool,
}

impl<P: Param> HandleResult<P> {
    pub fn add_all(&mut self, method: &str, param: P) {
        self.all.push((method.to_owned(), param));
    }

    pub fn add_one(&mut self, account: PeerId, method: &str, param: P) {
        self.one.push((account, method.to_owned(), param));
    }
}

/// serialize & deserialize for params
pub trait Param: Sized + Send + Default {
    fn to_value(self) -> Value;

    fn from_value(value: Value) -> Result<Self>;

    fn to_bytes(&self) -> Vec<u8>;

    fn from_bytes(bytes: &[u8]) -> Result<Self>;
}

#[async_trait::async_trait]
pub trait Handler: Send {
    type Param: Param;

    /// when player online
    async fn online(&mut self, _player: PeerId) -> Result<HandleResult<Self::Param>> {
        Ok(HandleResult::default())
    }

    /// when player offline
    async fn offline(&mut self, _player: PeerId) -> Result<HandleResult<Self::Param>> {
        Ok(HandleResult::default())
    }

    /// create new room scan from chain
    async fn create(peers: &[PeerId]) -> Self;

    /// handle message in a room
    async fn handle(
        &mut self,
        player: PeerId,
        method: &str,
        param: Self::Param,
    ) -> Result<HandleResult<Self::Param>>;
}

#[async_trait::async_trait]
pub trait Task {
    type H: Handler;

    fn timer(&self) -> u64;

    async fn run(
        &mut self,
        states: &mut Self::H,
    ) -> Result<HandleResult<<Self::H as Handler>::Param>>;
}
