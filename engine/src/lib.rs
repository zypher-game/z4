#[macro_use]
extern crate tracing;

mod config;
mod contracts;
mod engine;
mod p2p;
mod pool;
mod room;
mod rpc;
mod scan;

/// Module for ws/http/p2p request with channel.
#[cfg(feature = "request")]
pub mod request;

/// Z4 main config.
pub use config::Config;

/// Z4 main contracts.
pub use contracts::{RoomMarket, SimpleGame, Token};

/// Z4 main engine.
pub use engine::Engine;

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
pub use z4_types::*;

use serde::{Deserialize, Serialize};

/// P2P network message type
#[derive(Serialize, Deserialize)]
struct P2pMessage<'a> {
    pub method: &'a str,
    pub params: Vec<u8>,
}

/// The message type synced from chain
pub enum ChainMessage {
    /// create a room on the chain,
    /// room_id, game_id, viewable, player account, player peer id, player pubkey,
    /// salt by player, current block prevrandao
    CreateRoom(
        RoomId,
        GameId,
        bool,
        Address,
        PeerId,
        [u8; 32],
        [u8; 32],
        [u8; 32],
    ),
    /// join a room on the chain,
    /// room_id, player account, player peer id, player pubkey
    JoinRoom(RoomId, Address, PeerId, [u8; 32]),
    /// start a room on the chain,
    /// room_id, game address
    StartRoom(RoomId, Address),
    /// accept a room on the chain,
    /// room_id, sequencer account, sequencer websocket, params when accept
    AcceptRoom(RoomId, PeerId, String, Vec<u8>),
    /// game over in local,
    /// room_id, result, proof
    GameOverRoom(RoomId, Vec<u8>, Vec<u8>),
    /// game over on the chain,
    /// room_id
    ChainOverRoom(RoomId),
    /// need reprove in local
    Reprove,
}

/// The message type when send to pool
pub enum PoolMessage {
    /// send a transaction to accept room,
    /// room_id, params
    AcceptRoom(RoomId, Vec<u8>),
    /// send a transaction to over room,
    /// room_id, result, proof
    OverRoom(RoomId, Vec<u8>, Vec<u8>),
    /// the transaction had been submmited
    Submitted(RoomId),
}
