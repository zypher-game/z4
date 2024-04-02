#[macro_use]
extern crate tracing;

mod config;
mod contracts;
mod engine;
mod key;
mod p2p;
mod pool;
mod room;
mod rpc;
mod scan;
mod task;
mod types;

#[cfg(feature = "request")]
pub mod request;

pub use config::Config;
pub use contracts::*;
pub use engine::Engine;
pub use key::*;
pub use scan::chain_channel;
use serde::{Deserialize, Serialize};
pub use serde_json::{json, Value};
pub use tdn::{
    prelude::{GroupId, Peer, PeerId, PeerKey, SendType},
    types::rpc::rpc_response,
};
pub use types::*;

/// The result when after handling the message or task.
#[derive(Default, Serialize, Deserialize)]
pub struct HandleResult<P: Param> {
    /// need broadcast the msg in the room
    all: Vec<(String, P)>,
    /// need send to someone msg
    one: Vec<(PeerId, String, P)>,
    /// when game over, need prove the operations & states
    over: Option<(Vec<u8>, Vec<u8>)>,
}

impl<P: Param> HandleResult<P> {
    pub fn add_all(&mut self, method: &str, param: P) {
        self.all.push((method.to_owned(), param));
    }

    pub fn add_one(&mut self, account: PeerId, method: &str, param: P) {
        self.one.push((account, method.to_owned(), param));
    }

    pub fn over(&mut self, data: Vec<u8>, proof: Vec<u8>) {
        self.over = Some((data, proof));
    }

    pub fn replace_over(&mut self) -> Option<(Vec<u8>, Vec<u8>)> {
        if let Some((data, proof)) = &mut self.over {
            let d = std::mem::take(data);
            let p = std::mem::take(proof);
            Some((d, p))
        } else {
            None
        }
    }
}

/// serialize & deserialize for params
pub trait Param: Sized + Send + Default {
    fn to_value(self) -> Value;

    fn from_value(value: Value) -> Result<Self>;

    fn to_bytes(&self) -> Vec<u8>;

    fn from_bytes(bytes: &[u8]) -> Result<Self>;
}

/// Timer tasks when game room started
#[async_trait::async_trait]
pub trait Task: Send + Sync {
    type H: Handler;

    fn timer(&self) -> u64;

    async fn run(
        &mut self,
        state: &mut Self::H,
    ) -> Result<HandleResult<<Self::H as Handler>::Param>>;
}

/// Type helper for tasks
pub type Tasks<H> = Vec<Box<dyn Task<H = H>>>;

/// Handle message received from players
#[async_trait::async_trait]
pub trait Handler: Send + Sized + 'static {
    type Param: Param;

    /// accept params when submit to chain
    async fn accept(subgame: &SubGame, peers: &[(Address, PeerId, [u8; 32])]) -> Result<Vec<u8>>;

    /// when player online
    async fn online(&mut self, _player: PeerId) -> Result<HandleResult<Self::Param>> {
        Ok(HandleResult::default())
    }

    /// when player offline
    async fn offline(&mut self, _player: PeerId) -> Result<HandleResult<Self::Param>> {
        Ok(HandleResult::default())
    }

    /// create new room scan from chain
    async fn create(
        rid: RoomId,
        subgame: &SubGame,
        peers: &[(Address, PeerId, [u8; 32])],
        params: Vec<u8>,
    ) -> Result<(Self, Tasks<Self>)>;

    /// handle message in a room
    async fn handle(
        &mut self,
        player: PeerId,
        method: &str,
        param: Self::Param,
    ) -> Result<HandleResult<Self::Param>>;
}

/// Default vector json values for Param
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct DefaultParams(pub Vec<Value>);

impl Param for DefaultParams {
    fn to_value(self) -> Value {
        Value::Array(self.0)
    }

    fn from_value(value: Value) -> Result<Self> {
        match value {
            Value::Array(p) => Ok(DefaultParams(p)),
            o => Ok(DefaultParams(vec![o])),
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(&self.0).unwrap_or(vec![])
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let v: Value = serde_json::from_slice(bytes)?;
        Self::from_value(v)
    }
}
