mod error;
mod key;
mod network;
mod task;
mod utils;

use serde::{Deserialize, Serialize};
pub use serde_json::{json, Value};

pub use error::Error;
pub use ethereum_types::{Address, H160};
pub use key::*;
pub use network::*;
pub use task::*;
pub use tdn_types::primitives::PeerId;
pub use utils::*;

/// Z4 main Result with Z4 error
pub type Result<T> = core::result::Result<T, Error>;

/// The result when after handling the message or task.
#[derive(Default)]
pub struct HandleResult<P: Param> {
    /// Need broadcast the msg in the room
    pub all: Vec<P>,
    /// Need send to someone msg
    pub one: Vec<(PeerId, P)>,
    /// When game over, need prove the operations & states
    pub over: bool,
    /// When need waiting others, can use started = false (for PoZK)
    pub started: bool,
}

impl<P: Param> HandleResult<P> {
    /// Broadcast message to all players in the room
    pub fn add_all(&mut self, param: P) {
        self.all.push(param);
    }

    /// Send message to player in the room
    pub fn add_one(&mut self, account: PeerId, param: P) {
        self.one.push((account, param));
    }

    /// Over the room/game
    pub fn over(&mut self) {
        self.over = true;
    }

    /// Start the room/game, no other players can join
    pub fn started(&mut self) {
        self.started = true;
    }
}

/// Serialize & deserialize for params
pub trait Param: Sized + Send + Default {
    /// To json value
    fn to_string(&self) -> String;

    /// From json value
    fn from_string(s: String) -> Result<Self>;

    /// To bytes
    fn to_bytes(&self) -> Vec<u8>;

    /// From bytes
    fn from_bytes(bytes: Vec<u8>) -> Result<Self>;

    /// To json value
    fn to_value(&self) -> Value;

    /// From json value
    fn from_value(v: Value) -> Result<Self>;
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

/// Standard player from chain & pozk
pub struct Player {
    pub account: Address,
    pub peer: PeerId,
    pub signer: [u8; 32],
}

/// Player bytes length
pub const PLAYER_BYTES_LEN: usize = 72;

impl Player {
    /// deserialize Player from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Player> {
        if bytes.len() < PLAYER_BYTES_LEN {
            return Err(Error::Serialize);
        }

        let mut account_bytes = [0u8; 20];
        let mut peer_bytes = [0u8; 20];
        let mut signer_bytes = [0u8; 32];
        account_bytes.copy_from_slice(&bytes[0..20]);
        peer_bytes.copy_from_slice(&bytes[20..40]);
        signer_bytes.copy_from_slice(&bytes[40..72]);

        Ok(Player {
            account: H160(account_bytes),
            peer: PeerId(peer_bytes),
            signer: signer_bytes,
        })
    }

    /// serialize Player to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = self.account.0.to_vec();
        bytes.extend(self.peer.0.to_vec());
        bytes.extend(self.signer.to_vec());
        bytes
    }
}

/// Handle message received from players
#[async_trait::async_trait]
pub trait Handler: Send + Sized + 'static {
    /// Request/Response params
    type Param: Param;

    /// Viewable for game
    fn viewable() -> bool {
        false
    }

    /// Accept params when submit to chain
    async fn chain_accept(_players: &[Player]) -> Vec<u8> {
        vec![]
    }

    /// Create new room scan from chain
    async fn chain_create(
        _players: &[Player],
        _params: Vec<u8>,
        _rid: RoomId,
        _seed: [u8; 32],
    ) -> Option<(Self, Tasks<Self>)> {
        None
    }

    /// Create new room from PoZK
    async fn pozk_create(
        _player: Player,
        _params: Vec<u8>,
        _rid: RoomId,
    ) -> Option<(Self, Tasks<Self>)> {
        None
    }

    /// New player join from PoZK
    async fn pozk_join(
        &mut self,
        _player: Player,
        _params: Vec<u8>,
    ) -> Result<HandleResult<Self::Param>> {
        Ok(HandleResult::default())
    }

    /// New Viewer online if viewable is true
    async fn viewer_online(&mut self, _peer: PeerId) -> Result<HandleResult<Self::Param>> {
        Ok(HandleResult::default())
    }

    /// New Viewer offline if viewable is true
    async fn viewer_offline(&mut self, _peer: PeerId) -> Result<HandleResult<Self::Param>> {
        Ok(HandleResult::default())
    }

    /// When player online
    async fn online(&mut self, _peer: PeerId) -> Result<HandleResult<Self::Param>> {
        Ok(HandleResult::default())
    }

    /// When player offline
    async fn offline(&mut self, _peer: PeerId) -> Result<HandleResult<Self::Param>> {
        Ok(HandleResult::default())
    }

    /// Handle message in a room
    async fn handle(
        &mut self,
        _peer: PeerId,
        _param: Self::Param,
    ) -> Result<HandleResult<Self::Param>> {
        Ok(HandleResult::default())
    }

    /// Generate proof for this game result, when find game is over
    async fn prove(&mut self) -> Result<(Vec<u8>, Vec<u8>)>;
}

impl Param for Value {
    fn to_string(&self) -> String {
        serde_json::to_string(&self).unwrap_or("".to_owned())
    }

    fn from_string(s: String) -> Result<Self> {
        Ok(serde_json::from_str(&s)?)
    }

    fn to_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(&self).unwrap_or(vec![])
    }

    fn from_bytes(bytes: Vec<u8>) -> Result<Self> {
        Ok(serde_json::from_slice(&bytes)?)
    }

    fn to_value(&self) -> Value {
        self.clone()
    }

    fn from_value(v: Value) -> Result<Self> {
        Ok(v)
    }
}

impl Param for String {
    fn to_string(&self) -> String {
        self.clone()
    }

    fn from_string(s: String) -> Result<Self> {
        Ok(s)
    }

    fn to_bytes(&self) -> Vec<u8> {
        self.as_bytes().to_vec()
    }

    fn from_bytes(bytes: Vec<u8>) -> Result<Self> {
        String::from_utf8(bytes).map_err(|_| Error::Serialize)
    }

    fn to_value(&self) -> Value {
        Value::String(self.clone())
    }

    fn from_value(v: Value) -> Result<Self> {
        v.as_str().map(|v| v.to_owned()).ok_or(Error::Serialize)
    }
}

impl Param for Vec<u8> {
    fn to_string(&self) -> String {
        hex::encode(&self)
    }

    fn from_string(s: String) -> Result<Self> {
        Ok(hex::decode(s)?)
    }

    fn to_bytes(&self) -> Vec<u8> {
        self.clone()
    }

    fn from_bytes(bytes: Vec<u8>) -> Result<Self> {
        Ok(bytes)
    }

    fn to_value(&self) -> Value {
        Value::String(self.to_string())
    }

    fn from_value(v: Value) -> Result<Self> {
        let s = v.as_str().map(|v| v.to_owned()).ok_or(Error::Serialize)?;
        Self::from_string(s)
    }
}

/// method & multiple values for Param, compatible with jsonrpc
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct MethodValues {
    pub method: String,
    pub params: Vec<Value>,
}

impl MethodValues {
    /// new a method with values params
    pub fn new(method: &str, params: Vec<Value>) -> Self {
        Self {
            method: method.to_owned(),
            params,
        }
    }
}

impl Param for MethodValues {
    fn to_string(&self) -> String {
        serde_json::to_string(&self).unwrap_or("".to_owned())
    }

    fn from_string(s: String) -> Result<Self> {
        Ok(serde_json::from_str(&s)?)
    }

    fn to_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(&self).unwrap_or(vec![])
    }

    fn from_bytes(bytes: Vec<u8>) -> Result<Self> {
        Ok(serde_json::from_slice(&bytes)?)
    }

    fn to_value(&self) -> Value {
        json!({
            "method": self.method,
            "params": self.params,
        })
    }

    fn from_value(v: Value) -> Result<Self> {
        Ok(serde_json::from_value(v)?)
    }
}
