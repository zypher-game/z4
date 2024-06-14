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

/// Module for ws/http/p2p request with channel.
#[cfg(feature = "request")]
pub mod request;

/// Z4 main config.
pub use config::Config;

/// Z4 main contracts.
pub use contracts::{simple_game_result, Network, NetworkConfig, RoomMarket, SimpleGame, Token};

/// Z4 main engine.
pub use engine::Engine;

/// Z4 keypair.
pub use key::*;

/// Create z4 scan(sync from chain) channel.
pub use scan::chain_channel;

/// Export serde_json's json and Value.
pub use serde_json::{json, Value};

/// Export useful tdn core struct and functions.
pub use tdn::{
    prelude::{GroupId, Peer, PeerId, PeerKey, SendType},
    types::rpc::rpc_response,
};

/// Export useful types
pub use types::*;

/// The result when after handling the message or task.
#[derive(Default)]
pub struct HandleResult<P: Param> {
    /// Need broadcast the msg in the room
    all: Vec<(String, P)>,
    /// Need send to someone msg
    one: Vec<(PeerId, String, P)>,
    /// When game over, need prove the operations & states
    over: Option<(Vec<u8>, Vec<u8>)>,
}

impl<P: Param> HandleResult<P> {
    /// Broadcast message to all players in the room
    pub fn add_all(&mut self, method: &str, param: P) {
        self.all.push((method.to_owned(), param));
    }

    /// Send message to player in the room
    pub fn add_one(&mut self, account: PeerId, method: &str, param: P) {
        self.one.push((account, method.to_owned(), param));
    }

    /// Over the room/game
    pub fn over(&mut self, data: Vec<u8>, proof: Vec<u8>) {
        self.over = Some((data, proof));
    }

    /// Do some replace with over params
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

/// Serialize & deserialize for params
pub trait Param: Sized + Send + Default {
    /// To json value
    fn to_value(self) -> Value;

    /// From json value
    fn from_value(value: Value) -> Result<Self>;

    /// To bytes
    fn to_bytes(&self) -> Vec<u8>;

    /// From bytes
    fn from_bytes(bytes: &[u8]) -> Result<Self>;
}

/// Timer tasks when game room started
#[async_trait::async_trait]
pub trait Task: Send + Sync {
    /// Game logic Handler
    type H: Handler;

    /// Next time for execute the task
    fn timer(&self) -> u64;

    /// Execute the task
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
    /// Request/Response params
    type Param: Param;

    /// Accept params when submit to chain
    async fn accept(_peers: &[(Address, PeerId, [u8; 32])]) -> Vec<u8> {
        vec![]
    }

    /// When player online
    async fn online(&mut self, _player: PeerId) -> Result<HandleResult<Self::Param>> {
        Ok(HandleResult::default())
    }

    /// When player offline
    async fn offline(&mut self, _player: PeerId) -> Result<HandleResult<Self::Param>> {
        Ok(HandleResult::default())
    }

    /// Create new room scan from chain
    async fn create(
        peers: &[(Address, PeerId, [u8; 32])],
        params: Vec<u8>,
        rid: RoomId,
        seed: [u8; 32],
    ) -> (Self, Tasks<Self>);

    /// Handle message in a room
    async fn handle(
        &mut self,
        player: PeerId,
        method: &str,
        param: Self::Param,
    ) -> Result<HandleResult<Self::Param>>;
}

/// Default vector json values for Param
#[derive(Default, Debug, Clone)]
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
