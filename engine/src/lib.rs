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
pub struct HandleResult {
    /// need broadcast the msg in the room
    all: Vec<(String, Vec<Value>)>,
    /// need send to someone msg
    one: Vec<(PeerId, String, Vec<Value>)>,
    /// when game over, need prove the operations & states
    over: bool,
}

impl HandleResult {
    pub fn add_all(&mut self, method: &str, params: Vec<Value>) {
        self.all.push((method.to_owned(), params));
    }

    pub fn add_one(&mut self, account: PeerId, method: &str, params: Vec<Value>) {
        self.one.push((account, method.to_owned(), params));
    }
}

#[async_trait::async_trait]
pub trait Handler: Send {
    /// when player online
    async fn online(&mut self, _player: PeerId) -> Result<HandleResult> {
        Ok(HandleResult::default())
    }

    /// when player offline
    async fn offline(&mut self, _player: PeerId) -> Result<HandleResult> {
        Ok(HandleResult::default())
    }

    /// create new room scan from chain
    async fn create(peers: &[PeerId]) -> Self;

    /// handle message in a room
    async fn handle(
        &mut self,
        player: PeerId,
        method: &str,
        params: Vec<Value>,
    ) -> Result<HandleResult>;
}

#[async_trait::async_trait]
pub trait Task {
    type H: Handler;

    fn timer(&self) -> u64;

    async fn run(&mut self, states: &mut Self::H) -> Result<HandleResult>;
}
